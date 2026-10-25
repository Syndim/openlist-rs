use crate::client::{OpenListClient, RequestBuilderExt};
use crate::error::Result;
use crate::models::*;

pub struct AuthApi<'a> {
    pub(crate) client: &'a OpenListClient,
}

impl AuthApi<'_> {
    /// Authenticate with username and password, returns JWT token.
    pub async fn login(&self, req: &LoginRequest) -> Result<ApiResponse<LoginData>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/auth/login"))
            .json(req)
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Authenticate with username and pre-hashed (SHA256) password.
    pub async fn login_hash(&self, req: &LoginHashRequest) -> Result<ApiResponse<LoginData>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/auth/login/hash"))
            .json(req)
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Authenticate via LDAP directory service.
    pub async fn login_ldap(&self, req: &LdapLoginRequest) -> Result<ApiResponse<LoginData>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/auth/login/ldap"))
            .json(req)
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Invalidate current session token.
    pub async fn logout(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/auth/logout"))
            .headers(self.client.auth_headers())
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Generate a new 2FA (TOTP) secret for current user.
    pub async fn generate_2fa(&self) -> Result<ApiResponse<Generate2FAData>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/auth/2fa/generate"))
            .headers(self.client.auth_headers())
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Verify TOTP code and enable 2FA for current user.
    pub async fn verify_2fa(
        &self,
        req: &Verify2FARequest,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/auth/2fa/verify"))
            .headers(self.client.auth_headers())
            .json(req)
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Redirect URL for SSO login.
    pub async fn sso_login_redirect(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/auth/sso"))
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Handle SSO callback after authentication.
    pub async fn sso_callback(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/auth/sso_callback"))
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Initiate WebAuthn (FIDO2/Passkey) authentication flow.
    pub async fn begin_webauthn_login(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/authn/webauthn_begin_login"))
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Complete WebAuthn authentication with signed challenge.
    pub async fn finish_webauthn_login(
        &self,
        body: &serde_json::Value,
    ) -> Result<ApiResponse<LoginData>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/authn/webauthn_finish_login"))
            .json(body)
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Start registering a new WebAuthn credential (requires auth).
    pub async fn begin_webauthn_registration(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/authn/webauthn_begin_registration"))
            .headers(self.client.auth_headers())
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Complete WebAuthn credential registration.
    pub async fn finish_webauthn_registration(
        &self,
        body: &serde_json::Value,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/authn/webauthn_finish_registration"))
            .headers(self.client.auth_headers())
            .json(body)
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// Remove a registered WebAuthn credential.
    pub async fn delete_webauthn(
        &self,
        body: &serde_json::Value,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .post(self.client.url("/api/authn/delete_authn"))
            .headers(self.client.auth_headers())
            .json(body)
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }

    /// List all registered WebAuthn credentials for current user.
    pub async fn get_webauthn_credentials(&self) -> Result<ApiResponse<serde_json::Value>> {
        let resp = self
            .client
            .http
            .get(self.client.url("/api/authn/getcredentials"))
            .headers(self.client.auth_headers())
            .send_with(self.client)
            .await?;
        self.client.handle_response(resp).await
    }
}
