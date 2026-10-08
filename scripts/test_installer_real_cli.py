#!/usr/bin/env python3
"""Actual Qeli CLI integration in disposable Linux chroots; daemon/kernel are seams.

Set QELI_AUDIT_BINARY to an explicitly provenance-checked Linux binary. No download
or Rust build is performed; missing binary is a skip, never runtime qualification.
"""
import hashlib
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import unittest
import test_installer_secrets as fixture

BINARY = Path(os.environ.get('QELI_AUDIT_BINARY', '/nonexistent-qeli-audit-binary'))
RECORDS = os.environ.get('QELI_REAL_CLI_RECORDS')

@unittest.skipUnless(sys.platform == 'linux' and getattr(os, 'geteuid', lambda: -1)() == 0 and BINARY.is_file(), 'Linux root and explicit QELI_AUDIT_BINARY required')
class RealCliTests(unittest.TestCase):
    stub = fixture.SecretTests.__dict__['stub']
    tearDownClass = fixture.SecretTests.__dict__['tearDownClass']

    @classmethod
    def setUpClass(cls):
        fixture.SecretTests.setUpClass.__func__(cls)
        (cls.base / 'usr/local/bin/qeli').unlink()
        shutil.copy2(BINARY, cls.base / 'fixtures/new-qeli')
        (cls.base / 'fixtures/new-qeli').chmod(0o755)
        for lib in re.findall(r'(/[^\s]+)', subprocess.run(['ldd', str(BINARY)], capture_output=True, text=True).stdout):
            p=cls.base / lib.lstrip('/');p.parent.mkdir(parents=True, exist_ok=True);shutil.copy2(lib,p)

    def setUp(self):
        fixture.SecretTests.setUp(self)
        (self.root / 'etc/qeli/server.conf').unlink()
        (self.root / 'etc/qeli/users.conf').unlink()
        (self.root / 'etc/qeli/identity/default.key').unlink()
        self.env['QELI_RUN_AS']='root'

    def run_cli(self, *args, input=None):
        return subprocess.run([self.chroot,str(self.root),'/usr/bin/qeli',*args],env=self.env,input=input,capture_output=True,text=True,timeout=30)

    def install(self, profile):
        self.env['QELI_PROFILE']=profile
        p=subprocess.run([self.chroot,str(self.root),'/bin/bash','/installer.sh','vpn.example'],env=self.env,capture_output=True,text=True,timeout=60)
        if RECORDS:
            r=Path(RECORDS)/self.id().split('.')[-1];r.mkdir(parents=True,exist_ok=True)
            (r/'installer.log').write_text(p.stdout+p.stderr)
            for file in (self.root/'etc/qeli').rglob('*'):
                if file.is_file():
                    dst=r/file.relative_to(self.root/'etc/qeli');dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(file,dst)
            (r/'returncode').write_text(str(p.returncode))
        self.assertEqual(p.returncode,0,p.stdout+p.stderr)
        self.assertFalse(list((self.root/'etc/qeli').glob('*.qeli-*')))
        conf=self.root/'etc/qeli/server.conf';self.assertEqual(conf.stat().st_mode&0o777,0o600);self.assertEqual(conf.stat().st_uid,234)
        users=(self.root/'etc/qeli/users.conf').read_text();self.assertEqual(users.count('[user:phone'),5);self.assertEqual(users.count('$argon2'),5)
        text=conf.read_text();self.assertIn('[profile:'+profile+']',text);self.assertEqual(text.count('[profile:'),1);self.assertIn('$argon2',text)
        self.assertEqual(self.run_cli('check-config','--config','/etc/qeli/server.conf').returncode,0)
        summary=(self.root/'etc/qeli/client-links/CONNECTION-STRINGS.txt').read_text();self.assertEqual(summary.count('qeli://'),5)
        self.assertFalse(list((self.root/'etc/qeli/client-links').glob('INSTALL-PENDING-*')))
        return text

    def test_actual_udp_installer(self): self.install('udp-quic')
    def test_actual_fake_tls_installer(self): self.install('fake-tls')
    def test_actual_reality_installer(self): self.install('reality-tls')

    def test_actual_invalid_stage_refuses_without_changing_live(self):
        self.install('udp-quic');live=self.root/'etc/qeli/server.conf';old=live.read_bytes();stage=live.with_name('server.conf.qeli-invalid')
        stage.write_text(live.read_text().replace('bind.port = 443','bind.port = invalid'))
        p=self.run_cli('check-config','--config','/etc/qeli/server.conf.qeli-invalid');self.assertNotEqual(p.returncode,0,p.stdout+p.stderr);self.assertEqual(live.read_bytes(),old)
        stage.unlink()

    def test_actual_custom_service_override_and_revert(self):
        shutil.copy2(BINARY,self.root/'usr/bin/qeli');(self.root/'usr/bin/qeli').chmod(0o755)
        for user in ['root','qeli']:
            p=self.run_cli('set-service-user',user,'--unit','audit-fixture.service');self.assertEqual(p.returncode,0,p.stderr)
            path=self.root/'etc/systemd/system/audit-fixture.service.d/run-as.conf'
            if user=='root':self.assertIn('User=root',path.read_text())
            else:self.assertFalse(path.exists())
        self.assertNotIn('restart', (self.root/'records/systemctl.log').read_text())

if __name__=='__main__':unittest.main(verbosity=2)
