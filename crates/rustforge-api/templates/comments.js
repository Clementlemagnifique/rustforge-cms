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
