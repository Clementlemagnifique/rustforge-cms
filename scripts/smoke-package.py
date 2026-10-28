"""End-to-end smoke test of an extracted package using an explicitly disposable database."""
import argparse, hashlib, json, os, re, subprocess, time, tomllib, urllib.request, urllib.error
from pathlib import Path
from urllib.parse import urlsplit

p=argparse.ArgumentParser();p.add_argument('--package',type=Path,required=True);p.add_argument('--port',type=int,default=55488);p.add_argument('--hold',action='store_true');args=p.parse_args()
root=args.package.resolve();db=os.environ.get('RUSTFORGE_SMOKE_DATABASE_URL','')
assert urlsplit(db).path.startswith('/rf_v1_smoke_'),'Use an explicitly disposable rf_v1_smoke_* database'
assert not (root/'config.toml').exists(),'Extract the package into a new folder'
binary=root/('rustforge.exe' if os.name=='nt' else 'rustforge')
env=os.environ.copy()
for key in ['JWT_SECRET','HOST','PORT','UPLOADS_DIR','PLUGINS_DIR','THEMES_DIR','ADMIN_DIST','ACTIVE_THEME','PROTECTED_DOWNLOADS_DIR']:
    env.pop(key,None)
env['DATABASE_URL']=db;env['RUSTFORGE_SETUP_TOKEN']='smoke-only-setup-token-0123456789-abcdefghijklmnopqrstuvwxyz'
flags=subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0
subprocess.run([str(binary),'init'],cwd=root,env=env,check=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,creationflags=flags)
config=(root/'config.toml').read_bytes();parsed=tomllib.loads(config.decode());assert len(parsed['server']['jwt_secret'])>=32
retry=subprocess.run([str(binary),'init'],cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,creationflags=flags)
assert retry.returncode!=0 and (root/'config.toml').read_bytes()==config,'Existing configuration was changed'
log=(root/'smoke-server.log').open('wb')
server=subprocess.Popen([str(binary),'serve','--host','127.0.0.1','--port',str(args.port)],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,creationflags=flags)
base=f'http://127.0.0.1:{args.port}';token=None;checks=['init and no-overwrite'];results={}
def request(path,data=None,method=None,ctype='application/json'):
    headers={'Content-Type':ctype}
    if token:headers['Authorization']='Bearer '+token
    payload=json.dumps(data).encode() if isinstance(data,dict) else data
    req=urllib.request.Request(base+path,data=payload,headers=headers,method=method)
    try:
        with urllib.request.build_opener(urllib.request.ProxyHandler({})).open(req,timeout=10) as response:return response.status,response.read(),response.headers
    except urllib.error.HTTPError as e:return e.code,e.read(),e.headers
def call(path,data=None):
    code,body,_=request(path,data);assert code==200,(path,code,body[:300]);return json.loads(body)
try:
    for _ in range(100):
        if server.poll() is not None:raise RuntimeError('Packaged server exited during startup; inspect smoke-server.log')
        try:
            if request('/api/v1/setup/status')[0]==200:break
        except OSError:pass
        time.sleep(.1)
    else:raise RuntimeError('Server did not start')
    assert call('/api/v1/setup/status')['is_installed'] is False
    installed=call('/api/v1/setup/install',{'setup_token':env['RUSTFORGE_SETUP_TOKEN'],'site_title':'TrustPress · recette V1','site_url':base,'admin_email':'testeur@example.test','admin_username':'testeur','admin_password':'TrustPress-test-2026!'})
    token=installed['token'];checks.append('setup')
    login=call('/api/v1/auth/login',{'login':'testeur','password':'TrustPress-test-2026!' });assert login['token'];checks.append('login')
    created=call('/api/v1/entries?content_type=page',{'title':'Bienvenue sur votre site','slug':'bienvenue-v1','body':'# Un site à votre image\n\nUne page **publiée** avec TrustPress.','status':'published'})
    assert 'Un site à votre image' in request('/fr/page/bienvenue-v1')[1].decode();checks.append('published page -> public HTML')
    preview=call('/api/v1/entries/preview',{'body':'**Gras** <script>alert(1)</script>'});assert '<strong>Gras</strong>' in preview['body_html'] and '<script' not in preview['body_html'];checks.append('safe preview')
    _,html,_=request('/admin/');asset=re.search(r'src="([^"]+\.js)"',html.decode()).group(1);assert asset.startswith('/admin/assets/') and request(asset)[0]==200;checks.append('Studio assets')
    assert request('/api/v1/capsules/export')[0]==200;checks.append('native backup')
    sample=(root/'exemples/wordpress-editorial.xml').read_bytes()
    def migration(path,fields):
        boundary='trustpress-package-smoke'
        body=(f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="export.xml"\r\nContent-Type: text/xml\r\n\r\n').encode()+sample+b'\r\n'
        for key,value in fields.items():body+=f'--{boundary}\r\nContent-Disposition: form-data; name="{key}"\r\n\r\n{value}\r\n'.encode()
        body+=f'--{boundary}--\r\n'.encode()
        code,data,_=request(path,body,ctype=f'multipart/form-data; boundary={boundary}')
        assert code==200,(path,code,data[:300]);return json.loads(data)
    inspected=migration('/api/v1/converter/inspect-wxr',{});assert inspected['entries']==2
    for _ in range(2):migration('/api/v1/converter/import-wxr',{'seal':inspected['seal'],'acknowledge':'true'})
    checks.append('inspect -> confirmed WordPress import -> repeat without duplicate')
    # Let the actual background scheduler tick, rather than checking only the immediate result.
    time.sleep(31.5)
    pages=call('/api/v1/entries?content_type=page')['items'];private=next(e for e in pages if e['slug']=='presentation')
    assert private['status']=='draft' and private['published_at'] is None
    assert request('/fr/page/presentation')[0]==404 and 'Présentation privée' not in request('/fr')[1].decode()
    checks.append('private WordPress page remains private after real scheduler tick')
    results={'status':'passed','checks':checks,'base_url':base,'page_id':created['id'],'login':'testeur','test_password':'TrustPress-test-2026!'}
    (root/'smoke-result.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps({'status':'passed','checks':checks,'browser_url':base+'/admin/','hold':args.hold}),flush=True)
    if args.hold:
        deadline=time.monotonic()+1200
        while not (root/'STOP').exists() and time.monotonic()<deadline:time.sleep(.5)
finally:
    server.terminate()
    try:server.wait(timeout=10)
    except subprocess.TimeoutExpired:server.kill();server.wait()
    log.close()
