/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{net::SocketAddr, sync::Mutex};
// Third Party Imports
use pyo3::prelude::*;
// Local Imports
use crate::to_value_error;
use dokime_server::grpc::{
    Config, TlsIdentity,
    access::{BearerGrant, Role},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[pyclass(name = "Role", module = "dokime.runtime", frozen, from_py_object)]
#[derive(Clone, PartialEq)]
pub struct PyRole {
    inner: Role,
}

#[pyclass(name = "BearerGrant", module = "dokime.runtime")]
pub struct PyBearerGrant {
    inner: BearerGrant,
}

#[pyclass(name = "TlsIdentity", module = "dokime.runtime")]
pub struct PyTlsIdentity {
    inner: Mutex<Option<TlsIdentity>>,
}

#[pyclass(name = "ServerConfig", module = "dokime.runtime")]
pub struct PyServerConfig {
    inner: Mutex<Option<Config>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[pymethods]
impl PyRole {
    #[classattr]
    #[allow(non_snake_case)]
    fn Read() -> Self {
        Self { inner: Role::Read }
    }

    #[classattr]
    #[allow(non_snake_case)]
    fn Control() -> Self {
        Self {
            inner: Role::Control,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    fn Admin() -> Self {
        Self { inner: Role::Admin }
    }

    fn __repr__(&self) -> &'static str {
        match self.inner {
            Role::Read => "Role.Read",
            Role::Control => "Role.Control",
            Role::Admin => "Role.Admin",
        }
    }

    fn __int__(&self) -> i32 {
        match self.inner {
            Role::Read => 0,
            Role::Control => 1,
            Role::Admin => 2,
        }
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(role) = other.extract::<PyRef<'_, Self>>() {
            return self.inner == role.inner;
        }
        other
            .extract::<i32>()
            .is_ok_and(|value| self.__int__() == value)
    }
}

#[pymethods]
impl PyBearerGrant {
    #[new]
    fn new(subject: String, role: PyRole, token: &str) -> PyResult<Self> {
        let role = role.inner;
        Ok(Self {
            inner: BearerGrant::new(subject, role, token).map_err(to_value_error)?,
        })
    }
    fn __repr__(&self) -> &'static str {
        "BearerGrant(<redacted>)"
    }
}

#[pymethods]
impl PyTlsIdentity {
    #[staticmethod]
    fn from_pem(certificate: Vec<u8>, private_key: Vec<u8>) -> Self {
        Self {
            inner: Mutex::new(Some(TlsIdentity::from_pem(certificate, private_key))),
        }
    }
    fn __repr__(&self) -> &'static str {
        "TlsIdentity(<redacted>)"
    }
}

#[pymethods]
impl PyServerConfig {
    #[staticmethod]
    #[pyo3(signature = (*, host = "127.0.0.1", port = 0, max_message_bytes = 1_048_576, grants = None))]
    fn local(
        py: Python<'_>,
        host: &str,
        port: u16,
        max_message_bytes: usize,
        grants: Option<Vec<Py<PyBearerGrant>>>,
    ) -> PyResult<Self> {
        let mut inner = Config::local()
            .with_bind_addr(address(host, port)?)
            .with_max_message_bytes(max_message_bytes);
        if let Some(grants) = grants {
            inner = inner
                .with_bearer_auth(clone_grants(py, grants))
                .map_err(to_value_error)?;
        }
        Ok(Self {
            inner: Mutex::new(Some(inner)),
        })
    }
    #[staticmethod]
    #[pyo3(signature = (identity, grants, *, host = "0.0.0.0", port = 0, max_message_bytes = 1_048_576))]
    fn remote(
        py: Python<'_>,
        identity: &PyTlsIdentity,
        grants: Vec<Py<PyBearerGrant>>,
        host: &str,
        port: u16,
        max_message_bytes: usize,
    ) -> PyResult<Self> {
        let address = address(host, port)?;
        let tls = identity
            .inner
            .lock()
            .expect("TLS identity poisoned")
            .take()
            .ok_or_else(|| to_value_error("TLS identity was already consumed"))?;
        let inner = Config::remote(address, tls, clone_grants(py, grants))
            .map_err(to_value_error)?
            .with_max_message_bytes(max_message_bytes);
        Ok(Self {
            inner: Mutex::new(Some(inner)),
        })
    }
    fn __repr__(&self) -> &'static str {
        "ServerConfig(<native configuration>)"
    }
}

impl PyServerConfig {
    pub(crate) fn take(&self) -> PyResult<Config> {
        self.inner
            .lock()
            .expect("server configuration poisoned")
            .take()
            .ok_or_else(|| to_value_error("server configuration was already consumed"))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn address(host: &str, port: u16) -> PyResult<SocketAddr> {
    Ok(SocketAddr::new(host.parse().map_err(to_value_error)?, port))
}

fn clone_grants(py: Python<'_>, grants: Vec<Py<PyBearerGrant>>) -> Vec<BearerGrant> {
    grants
        .into_iter()
        .map(|grant| grant.borrow(py).inner.clone())
        .collect()
}

/* === Tests === */
