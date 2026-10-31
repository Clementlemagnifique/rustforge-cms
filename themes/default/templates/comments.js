(function () {
    'use strict';
    var form = document.getElementById('comment-form');
    if (!form) return;
    var loginForm = document.getElementById('comment-login');
    var logout = document.getElementById('comment-logout');
    var account = document.getElementById('comment-account-state');
    var fields = document.getElementById('comment-fields');
    var identity = document.getElementById('comment-identity');
    var status = document.getElementById('comment-status-msg');
    var submit = document.getElementById('comment-submit-btn');
    var loginButton = loginForm.querySelector('button');
    var password = loginForm.elements.namedItem('password');
    var requiresLogin = fields.disabled;
    // Session de cet onglet uniquement : aucun cookie, stockage ou profil dans le HTML SSR.
    var token = null;
    var remember = document.getElementById('comment-remember');
    var identityKey = 'rf.comment.identity.v1';
    var identityAge = 30 * 24 * 60 * 60 * 1000;
    function forgetIdentity() {
        try { localStorage.removeItem(identityKey); } catch (_) { /* Storage facultatif. */ }
    }
    if (remember) {
        try {
            var saved = JSON.parse(localStorage.getItem(identityKey));
            if (saved && saved.version === 1 && saved.expires > Date.now() && saved.expires <= Date.now() + identityAge
                && typeof saved.name === 'string' && saved.name.length <= 100
                && typeof saved.email === 'string' && saved.email.length <= 254
                && typeof saved.url === 'string' && saved.url.length <= 2048
                && Object.keys(saved).every(function (key) { return ['version','expires','name','email','url'].includes(key); })) {
                ['author_name','author_email','author_url'].forEach(function (name, index) {
                    var input = form.elements.namedItem(name);
                    if (!input.value) input.value = [saved.name, saved.email, saved.url][index];
                });
                remember.checked = true;
            } else if (saved) { forgetIdentity(); }
        } catch (_) { forgetIdentity(); }
        remember.addEventListener('change', function () { if (!remember.checked) forgetIdentity(); });
    } else { forgetIdentity(); }

    var messages = {
        connected: 'Connecté pour commenter : ',
        required: 'Connectez-vous pour commenter.',
        guest: 'Vous pouvez commenter comme visiteur ou vous connecter.',
        expired: 'Votre session a expiré. Reconnectez-vous ; votre texte est conservé.',
        connecting: 'Connexion en cours…',
        sending: 'Envoi en cours…',
        saved: 'Votre commentaire a été enregistré. Sa publication dépend de la modération.',
        failed: 'La requête a échoué. Réessayez.',
        network: 'Connexion au serveur impossible. Votre texte est conservé.'
    };
    function session(nextToken, name) {
        token = nextToken;
        fields.disabled = requiresLogin && !token;
        identity.hidden = Boolean(token);
        identity.style.display = token ? 'none' : 'grid';
        identity.querySelectorAll('input').forEach(function (input) { input.disabled = Boolean(token); });
        loginForm.hidden = Boolean(token);
        loginForm.style.display = token ? 'none' : 'flex';
        logout.hidden = !token;
        account.textContent = token ? messages.connected + name : (requiresLogin ? messages.required : messages.guest);
    }
    async function request(url, payload, auth) {
        var headers = { 'Content-Type': 'application/json' };
        if (auth) headers.Authorization = 'Bearer ' + auth;
        var response = await fetch(url, { method: 'POST', headers: headers, body: JSON.stringify(payload), credentials: 'omit' });
        var data = await response.json().catch(function () { return null; });
        if (!response.ok) {
            var error = new Error(data && data.error && data.error.message || messages.failed);
            error.status = response.status;
            throw error;
        }
        return data;
    }
    loginForm.addEventListener('submit', async function (event) {
        event.preventDefault();
        if (loginButton.disabled) return;
        loginButton.disabled = true;
        account.textContent = messages.connecting;
        try {
            var data = await request('/api/v1/public/session/login', {
                login: loginForm.elements.namedItem('login').value,
                password: password.value
            });
            if (!data || typeof data.token !== 'string' || typeof data.display_name !== 'string') throw new Error(messages.failed);
            session(data.token, data.display_name);
            form.elements.namedItem('content').focus();
        } catch (error) {
            account.textContent = error.status ? error.message : messages.network;
        } finally {
            password.value = '';
            loginButton.disabled = false;
        }
    });
    logout.addEventListener('click', function () {
        session(null);
        loginForm.elements.namedItem('login').focus();
    });
    form.addEventListener('submit', async function (event) {
        event.preventDefault();
        if (fields.disabled || submit.disabled) return;
        var data = new FormData(form);
        var guestSubmission = !token;
        submit.disabled = true;
        status.textContent = messages.sending;
        try {
            await request('/api/v1/public/entries/' + form.dataset.entryId + '/comments', {
                author_name: data.get('author_name') || '',
                author_email: data.get('author_email') || '',
                author_url: data.get('author_url') || null,
                content: data.get('content'),
                website_url_hp: data.get('website_url_hp') || null
            }, token);
            if (remember && guestSubmission && !token) {
                if (remember.checked) {
                    try { localStorage.setItem(identityKey, JSON.stringify({
                        version:1, expires:Date.now()+identityAge,
                        name:String(data.get('author_name') || ''),
                        email:String(data.get('author_email') || ''),
                        url:String(data.get('author_url') || '')
                    })); } catch (_) { /* L'enregistrement du commentaire reste réussi. */ }
                } else { forgetIdentity(); }
            }
            status.textContent = messages.saved;
            form.elements.namedItem('content').value = '';
        } catch (error) {
            if (error.status === 401 && token) {
                session(null);
                status.textContent = messages.expired;
                loginForm.elements.namedItem('login').focus();
            } else {
                status.textContent = error.status ? error.message : messages.network;
            }
        } finally {
            submit.disabled = false;
        }
    });
})();
