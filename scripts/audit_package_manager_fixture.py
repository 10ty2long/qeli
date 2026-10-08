#!/usr/bin/env python3
"""Q33 real apt/dpkg plus a real manager, using owned filesystem and unit mapping.

Remote entrypoint: root, private mount+NET namespace, verified package/updater,
precreated owned unit link and network namespace. No HTTP or package dependency
bootstrap: dependency status is copied from the lab, qeli is removed from that copy.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--unit', required=True)
    args = parser.parse_args()
    work = args.root.resolve()
    assert work == args.root and work.parent == Path('/var/tmp')
    assert re.fullmatch(r'qeli-q33-pkg-[0-9]+', args.unit)
    unit = args.unit + '.service'
    root = work / 'root'
    root.mkdir()
    result = {'status': 'IN_PROGRESS', 'checks': [], 'commands': []}

    def save():
        (work / 'results.json').write_text(json.dumps(result, indent=2))

    def run(argv, label, timeout=120, required=True):
        process = subprocess.run(argv, capture_output=True, timeout=timeout)
        text = (process.stdout + process.stderr).decode(errors='replace')
        result['commands'].append({'label': label, 'argv': argv, 'rc': process.returncode, 'output': text})
        save()
        if required:
            assert process.returncode == 0, (label, process.returncode, text[-3000:])
        return process.returncode, text

    def check(label, ok, detail=None):
        result['checks'].append({'name': label, 'status': 'PASS' if ok else 'FAIL', 'detail': detail})
        save()
        print(('PASS ' if ok else 'FAIL ') + label, flush=True)
        assert ok, (label, detail)

    def copy_file(source, target):
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)

    def executable(source, name=None):
        source = Path(source)
        copy_file(source, root / 'usr/bin' / (name or source.name))
        output = subprocess.run(['ldd', str(source)], capture_output=True, text=True).stdout
        for library in re.findall(r'(/[^\s]+)', output):
            copy_file(library, root / library.lstrip('/'))

    def chroot(argv, label, required=True):
        env = ['/usr/bin/env', 'PATH=/usr/local/bin:/usr/bin:/bin', 'LC_ALL=C',
               'DEBIAN_FRONTEND=noninteractive', 'SYSTEMD_IGNORE_CHROOT=1']
        return run(['chroot', str(root), *env, *argv], label, required=required)

    def prop(key):
        return run(['systemctl', 'show', unit, '-p', key, '--value'], key)[1].strip()

    def ready():
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            pid = prop('MainPID')
            if pid != '0':
                _, text = run(['nsenter', '-t', pid, '-n', '--', 'ss', '-lnt'], 'real unit listeners', required=False)
                if ':443 ' in text and ':24990 ' in text:
                    return pid
            time.sleep(.25)
        raise AssertionError('owned packaged service not ready')

    def state_snapshot():
        paths = ['server.conf', 'users.conf', 'identity/fixture.key',
                 'client-links/INSTALL-PENDING-fixture', 'server.conf.bak-fixture']
        return {name: {'sha256': hashlib.sha256((root/'etc/qeli'/name).read_bytes()).hexdigest(),
                       'uid': (root/'etc/qeli'/name).stat().st_uid,
                       'mode': stat.S_IMODE((root/'etc/qeli'/name).stat().st_mode)} for name in paths}

    try:
        run(['mount', '--make-rprivate', '/'], 'private mount propagation')
        for program in ['bash','env','id','date','awk','sed','sort','head','tail','grep','find','xargs',
                        'cut','cp','rm','install','stat','dirname','basename','mktemp','sha256sum','flock',
                        'dpkg','dpkg-deb','dpkg-query','tar','diff','ldconfig','start-stop-daemon',
                        'dpkg-split','chown','chmod','mkdir','cat','ln','getent','setcap','sleep',
                        'apt-get','perl','readlink','test']:
            executable(shutil.which(program))
        executable('/usr/bin/systemctl', 'systemctl.real')
        for folder in ['usr/share/debconf','usr/share/perl5/Debconf']:
            shutil.copytree('/'+folder, root/folder)
        for parent in ['/usr/share/perl','/usr/lib/x86_64-linux-gnu/perl','/usr/lib/x86_64-linux-gnu/perl-base']:
            if Path(parent).is_dir():
                shutil.copytree(parent, root/parent.lstrip('/'))
        shutil.copytree('/usr/lib/apt', root/'usr/lib/apt')
        for path in Path('/usr/lib/apt/methods').iterdir():
            if path.is_file():
                output = subprocess.run(['ldd', str(path)], capture_output=True, text=True).stdout
                for library in re.findall(r'(/[^\s]+)', output):
                    copy_file(library, root/library.lstrip('/'))
        for name in ['tmp','run/systemd/system','run/qeli-audit-unit','run/qeli-audit-dropin','etc/qeli',
                     'etc/systemd/system','var/lib/dpkg/info','var/lib/dpkg/updates','var/lib/dpkg/triggers',
                     'var/cache/debconf','var/cache/qeli','var/cache/apt/archives/partial','var/log/apt',
                     'var/lib/apt/lists/partial','etc/apt/sources.list.d','usr/local/bin','fixtures','dev','proc']:
            (root/name).mkdir(parents=True, exist_ok=True)
        (root/'bin').mkdir()
        for name in ['sh','bash']:
            (root/'bin'/name).symlink_to('../usr/bin/bash')
        for name, number in [('null',3),('zero',5),('random',8),('urandom',9)]:
            os.mknod(root/'dev'/name, stat.S_IFCHR | 0o666, os.makedev(1,number))
        for name in ['passwd','group']:
            rows=[line for line in Path('/etc/'+name).read_text().splitlines() if line.split(':')[0] in ['root','qeli','_apt']]
            (root/'etc'/name).write_text('\n'.join(rows)+'\n')
        copy_file('/etc/debconf.conf', root/'etc/debconf.conf')
        records=Path('/var/lib/dpkg/status').read_text().split('\n\n')
        records=[entry for entry in records if not re.search(r'^Package: qeli$', entry, re.M)]
        (root/'var/lib/dpkg/status').write_text('\n\n'.join(records)+'\n\n')
        (root/'etc/apt/sources.list').touch()
        (root/'etc/apt/apt.conf').write_text('APT::Get::List-Cleanup "false"; Acquire::Retries "0";\n')
        (root/'var/cache/qeli').chmod(0o700)
        # Bind only the manager socket and this test's exact owned unit/drop-in directories.
        (root/'run/systemd/private').touch()
        run(['mount','--bind','/run/systemd/private',str(root/'run/systemd/private')], 'real manager socket')
        for source, target in [(work/'unit-map',root/'run/qeli-audit-unit'),
                               (Path('/run/systemd/system')/(unit+'.d'),root/'run/qeli-audit-dropin')]:
            run(['mount','--bind',str(source),str(target)], 'owned manager paths')
        run(['mount','--bind','/proc',str(root/'proc')], 'read-only process inspection')
        run(['mount','-o','remount,bind,ro',str(root/'proc')], 'protect proc')
        wrapper = r'''#!/bin/bash
set -euo pipefail
render() {
 [ -f /lib/systemd/system/qeli.service ] || return 0
 awk -v name='UNITNAME' -v net='/run/netns/UNITNAME' -v binary='ROOTFS/usr/bin/qeli' -v conf='ROOTFS/etc/qeli' '
 /^StateDirectory=|^RuntimeDirectory=|^LogsDirectory=/ {sub(/=qeli$/, "="name)}
 /^\[Install\]/ {print "NetworkNamespacePath="net; print "BindReadOnlyPaths="binary":/usr/bin/qeli"; print "BindPaths="conf":/etc/qeli"; print "Environment=QELI_CONTROL_SOCKET=/run/"name"/control.sock"; print ""}
 {print}
 ' /lib/systemd/system/qeli.service > /run/qeli-audit-unit/UNITNAME.service.tmp
 mv /run/qeli-audit-unit/UNITNAME.service.tmp /run/qeli-audit-unit/UNITNAME.service
 if [ -f /etc/systemd/system/qeli.service.d/run-as.conf ] && [ ! -L /etc/systemd/system/qeli.service.d/run-as.conf ]; then
  cp /etc/systemd/system/qeli.service.d/run-as.conf /run/qeli-audit-dropin/run-as.conf
 else rm -f /run/qeli-audit-dropin/run-as.conf; fi
}
case "${1:-}" in
 daemon-reload) [ "$#" = 1 ] || exit 93; render ;;
 start|stop|restart|show|is-active|status|enable|disable)
  args=();found=no
  for arg in "$@"; do
   case "$arg" in qeli|qeli.service) args+=('UNITNAME.service'); found=yes ;; *) args+=("$arg") ;; esac
  done
  [ "$found" = yes ] || exit 94
  case "$1" in enable|disable) args+=(--runtime) ;; esac
  set -- "${args[@]}"
  ;;
 *) echo "unapproved manager operation" >&2;exit 95 ;;
esac
exec /usr/bin/systemctl.real "$@"
'''.replace('UNITNAME',args.unit).replace('ROOTFS',str(root))
        # Wrapper is a name/path adapter; it runs the genuine manager client and never fabricates health.
        (root/'usr/local/bin/systemctl').write_text(wrapper)
        (root/'usr/local/bin/systemctl').chmod(0o755)
        executable(shutil.which('mv'))
        copy_file(work/'update-qeli-server.sh',root/'update.sh')
        for revision in [1,2,3,4]:
            stage=work/('package-'+str(revision))
            run(['dpkg-deb','-R',str(work/'qeli_0.8.2_amd64.deb'),str(stage)], 'extract production package')
            control=stage/'DEBIAN/control'
            control.write_text(re.sub(r'^Version: .*$', 'Version: 0.8.2-'+str(revision),control.read_text(),flags=re.M))
            if revision==3:
                service=stage/'lib/systemd/system/qeli.service'
                service.write_text(service.read_text().replace('ExecStart=/usr/bin/qeli server --config /etc/qeli/server.conf','ExecStart=/usr/bin/qeli intentional-audit-startup-failure'))
            if revision==4:
                (stage/'usr/bin/qeli').write_bytes(b'not an ELF audit fault\n')
            package=root/'fixtures'/('qeli_0.8.2-'+str(revision)+'_amd64.deb')
            run(['dpkg-deb','--root-owner-group','-Zxz','--build',str(stage),str(package)], 'build revision/fault package',timeout=180)
        chroot(['/usr/bin/dpkg','-i','/fixtures/qeli_0.8.2-1_amd64.deb'],'fresh production maintainer scripts')
        check('fresh package configured by actual dpkg',chroot(['dpkg-query','-W','-f=${Status} ${Version}','qeli'],'fresh status')[1]=='install ok installed 0.8.2-1')
        (root/'etc/qeli/server.conf').write_text('''[auth]
users_file = /etc/qeli/users.conf
[web]
enabled = true
bind = 127.0.0.1
port = 24990
tls = false
secure_cookie = false
base_path = /audit
[profile:fixture]
bind.address = 127.0.0.1
bind.port = 443
bind.transport = tcp
tun.name = q33pkg
identity_key = /etc/qeli/identity/fixture.key
tun.address = 10.77.1.1
pool.cidr = 10.77.1.0/24
routing.forward_private = false
routing.nat.enabled = false
routing.ipv6.mode = off
dns.enabled = false
obf.mode = fake-tls
''')
        (root/'etc/qeli/server.conf').chmod(0o600)
        chroot(['qeli','set-web-password','--password','synthetic-package-panel-password','--config','/etc/qeli/server.conf'],'real panel password')
        chroot(['qeli','add-client','audit-package-user','--password','synthetic-package-user-password','--config','/etc/qeli/server.conf'],'real user persistence')
        links=root/'etc/qeli/client-links';links.mkdir(exist_ok=True);links.chmod(0o700)
        for path in [links/'INSTALL-PENDING-fixture',root/'etc/qeli/server.conf.bak-fixture']:
            path.write_text('synthetic administrator receipt\n');path.chmod(0o600)
        chroot(['qeli','set-service-user','qeli'],'real ownership/default run-as')
        chroot(['systemctl','start','qeli'],'real fresh package startup')
        pid=ready();check('fresh package supervisor and worker ready',pid!='0')
        baseline=state_snapshot();result['preserved_state']=baseline;save()
        shutil.copy2(root/'fixtures/qeli_0.8.2-1_amd64.deb',root/'var/cache/qeli/qeli_0.8.2-1_amd64.deb')
        (root/'var/cache/qeli/qeli_0.8.2-1_amd64.deb').chmod(0o600)
        for revision in [2,3,4]:
            rc,output=chroot(['env','QELI_DEB=/fixtures/qeli_0.8.2-'+str(revision)+'_amd64.deb','bash','/update.sh','--force'],'real updater revision'+str(revision),required=False)
            check('updater exit reflects revision'+str(revision),rc==(0 if revision==2 else 1),output[-1800:])
            current=chroot(['dpkg-query','-W','-f=${Status} ${Version}','qeli'],'installed state after revision'+str(revision))[1]
            check('exact package revision installed/recovered '+str(revision),current=='install ok installed 0.8.2-2',current)
            nextpid=ready();check('real service restarted/recovered '+str(revision),nextpid!=pid);pid=nextpid
            check('state bytes/owner/mode preserved '+str(revision),state_snapshot()==baseline)
            if revision!=2:
                check('failure reported as rollback '+str(revision),'rolled back' in output and 'Done —' not in output)
            check('real installed ELF restored '+str(revision),hashlib.sha256((root/'usr/bin/qeli').read_bytes()).hexdigest()==hashlib.sha256((work/'release-binary').read_bytes()).hexdigest())
        cache=root/'var/cache/qeli/qeli_0.8.2-2_amd64.deb'
        check('rollback cache private and exact package',cache.stat().st_uid==0 and stat.S_IMODE(cache.stat().st_mode)==0o600 and hashlib.sha256(cache.read_bytes()).digest()==hashlib.sha256((root/'fixtures/qeli_0.8.2-2_amd64.deb').read_bytes()).digest())
        chroot(['apt-get','remove','-y','qeli'],'real package remove')
        check('prerm stops actual owned service',prop('MainPID')=='0')
        check('remove preserves operator state',state_snapshot()==baseline)
        chroot(['dpkg','--purge','qeli'],'real package purge')
        check('purge preserves operator state',state_snapshot()==baseline)
        check('purge removes packaged examples',not(root/'etc/qeli/server.conf.example').exists())
        result['status']='PASS';save()
    except BaseException:
        import traceback
        result['status']='FAIL';result['error']=traceback.format_exc();save();raise
    finally:
        run(['systemctl','stop',unit],'stop only owned service',required=False)
        result['owned_main_pid_after_stop']=prop('MainPID')
        save()
    return 0


if __name__=='__main__':
    raise SystemExit(main())
