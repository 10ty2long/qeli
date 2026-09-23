#!/usr/bin/env python3
"""Bounded Linux worker lifecycle regression in fresh net/mount/PID namespaces."""
import argparse, json, os, signal, subprocess, sys, time
from pathlib import Path

def ns(kind):
    info=os.stat('/proc/self/ns/'+kind)
    return f'{info.st_dev}:{info.st_ino}'

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--qeli',required=True)
    ap.add_argument('--artifacts',required=True)
    ap.add_argument('--inside',action='store_true',help=argparse.SUPPRESS)
    args=ap.parse_args()
    binary=Path(args.qeli).resolve(strict=True)
    root=Path(args.artifacts).resolve()
    if not args.inside:
        if os.geteuid()!=0:raise RuntimeError('requires root in a disposable Linux lab')
        root.mkdir(mode=0o700,parents=True,exist_ok=False)
        env=dict(os.environ,QELI_AUDIT_PARENT_NET=ns('net'),QELI_AUDIT_PARENT_MNT=ns('mnt'))
        command=['unshare','--net','--mount','--pid','--fork','--kill-child=KILL','--mount-proc',sys.executable,str(Path(__file__).resolve()),'--qeli',str(binary),'--artifacts',str(root),'--inside']
        p=subprocess.run(command,env=env,timeout=420)
        return p.returncode
    if ns('net')==os.environ.get('QELI_AUDIT_PARENT_NET') or ns('mnt')==os.environ.get('QELI_AUDIT_PARENT_MNT') or not os.environ.get('QELI_AUDIT_PARENT_NET'):
        raise RuntimeError('fresh network and mount namespaces are mandatory')
    commands=[]
    def run(argv,check=True):
        p=subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=20)
        commands.append(dict(argv=argv,rc=p.returncode,output=p.stdout))
        (root/'commands.json').write_text(json.dumps(commands,indent=2))
        if check and p.returncode:raise RuntimeError(f'{argv}: {p.returncode}: {p.stdout}')
        return p
    run(['mount','--make-rprivate','/'])
    run(['mount','-t','sysfs','sysfs','/sys'])
    configdir=root/'etc-qeli';configdir.mkdir(mode=0o700)
    if not Path('/etc/qeli').is_dir():raise RuntimeError('/etc/qeli mount point required; host directory will not be created')
    run(['mount','--bind',str(configdir),'/etc/qeli'])
    run(['ip','link','set','lo','up'])
    run(['ip','link','add','wan0','type','dummy'])
    run(['ip','link','set','wan0','up'])
    run(['ip','addr','add','192.0.2.1/24','dev','wan0'])
    run(['ip','-6','addr','add','2001:db8::1/64','dev','wan0','nodad'])
    run(['ip','-6','route','add','default','dev','wan0'])
    (configdir/'users.conf').write_text('')
    results=[]
    for transport in ['tcp','udp']:
      for mode in ['off','manual','route','nat66']:
        case=root/(transport+'-'+mode);case.mkdir(mode=0o700)
        state=case/'state';state.mkdir(mode=0o700)
        runtime=case/'run';runtime.mkdir(mode=0o700)
        env=dict(os.environ,STATE_DIRECTORY=str(state),QELI_CONTROL_SOCKET=str(runtime/'control.sock'))
        cfg=configdir/'server.conf'
        # Each previous worker must already have released these settings.
        Path('/proc/sys/net/ipv4/ip_forward').write_text('0')
        Path('/proc/sys/net/ipv6/conf/all/forwarding').write_text('0')
        before_ra=Path('/proc/sys/net/ipv6/conf/wan0/accept_ra').read_text().strip()
        up=case/'up';down=case/'down'
        cfg.write_text(f"""[auth]
users_file = /etc/qeli/users.conf
[web]
enabled = false
[logging]
level = debug
[profile:audit]
identity_key = /etc/qeli/audit.key
bind.address = 127.0.0.1
bind.port = 24443
bind.transport = {transport}
tun.name = qeli-audit0
tun.address = 10.73.0.1
tun.ip_mode = dual
tun.ipv6_address = fd73::1
tun.queues = 1
pool.cidr = 10.73.0.0/24
pool.ipv6.cidr = fd73::/64
routing.nat.enabled = true
routing.nat.interface = wan0
routing.forward_private = true
routing.ipv6.mode = {mode}
routing.ipv6.interface = wan0
routing.ipv6.ndp_proxy = {'required' if mode=='manual' else 'off'}
routing.ipv6.ndp_proxy_interface = wan0
routing.post_up = printf up >> {up}
routing.post_down = printf down >> {down}
dns.enabled = false
obf.mode = fake-tls
""")
        cfg.chmod(0o600)
        run([str(binary),'check-config','-c',str(cfg)])
        good=cfg.read_text()
        cfg.write_text(good+'tun.mtu = broken\n')
        assert run([str(binary),'check-config','-c',str(cfg)],False).returncode!=0
        cfg.write_text(good)
        with (case/'worker.log').open('w') as log:
          worker=subprocess.Popen([str(binary),'_worker','-c',str(cfg)],env=env,stdout=log,stderr=subprocess.STDOUT)
          try:
            deadline=time.monotonic()+25
            while not up.exists():
              if worker.poll() is not None:raise RuntimeError('worker exited before ready: '+(case/'worker.log').read_text())
              if time.monotonic()>deadline:raise RuntimeError('worker readiness timeout: '+(case/'worker.log').read_text())
              time.sleep(.1)
            # Control listener comes up before profiles; hook proves this generation finished setup.
            assert Path('/proc/sys/net/ipv4/ip_forward').read_text().strip()=='1'
            assert Path('/proc/sys/net/ipv6/conf/all/forwarding').read_text().strip()==('1' if mode in ['route','nat66'] else '0')
            fw4=run(['iptables-save']).stdout;fw6=run(['ip6tables-save']).stdout
            (case/'firewall-active-v4.txt').write_text(fw4);(case/'firewall-active-v6.txt').write_text(fw6)
            assert 'qeli-nat:audit' in fw4
            assert ('qeli-nat:audit' in fw6)==(mode!='manual')
            second=subprocess.run([str(binary),'_worker','-c',str(cfg)],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=20)
            (case/'second-worker.log').write_text(second.stdout)
            assert second.returncode!=0 and 'already owned' in second.stdout,second.stdout
            assert worker.poll() is None and (runtime/'control.sock').exists()
            # Rejected reload must preserve the current generation and control listener.
            cfg.write_text(good+'tun.mtu = broken\n')
            worker.send_signal(signal.SIGHUP);time.sleep(.35)
            assert worker.poll() is None and up.read_text()=='up'
            run([str(binary),'list-clients','--socket',str(runtime/'control.sock')])
            cfg.write_text(good)
            worker.send_signal(signal.SIGTERM)
            assert worker.wait(timeout=20)==0,(case/'worker.log').read_text()
          finally:
            if worker.poll() is None:
              worker.kill();worker.wait(timeout=5)
        assert down.read_text()=='down', 'post_down did not run exactly once'
        assert not (runtime/'control.sock').exists(), 'control socket leaked'
        assert run(['ip','link','show','qeli-audit0'],False).returncode!=0,'TUN leaked'
        assert 'qeli-nat:audit' not in run(['iptables-save']).stdout,'IPv4 rules leaked'
        assert 'qeli-nat:audit' not in run(['ip6tables-save']).stdout,'IPv6 rules leaked'
        assert Path('/proc/sys/net/ipv4/ip_forward').read_text().strip()=='0','IPv4 forwarding lease leaked'
        assert Path('/proc/sys/net/ipv6/conf/all/forwarding').read_text().strip()=='0','IPv6 forwarding lease leaked'
        assert Path('/proc/sys/net/ipv6/conf/wan0/accept_ra').read_text().strip()==before_ra,'WAN accept_ra leaked'
        assert not (state/'sysctls.state').exists(),'journal ownership leaked'
        results.append(dict(transport=transport,mode=mode,status='PASS'))
        (root/'results.json').write_text(json.dumps(results,indent=2))
        print(transport,mode,'PASS',flush=True)
    return 0

if __name__=='__main__':raise SystemExit(main())
