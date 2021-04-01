extern crate futures;
extern crate tokio;

use super::{
    error::{Error, Kind, Operate},
    Submitter,
};
use crate::{account::Account, judge::Session};
use futures::future::join_all;
use std::vec::Vec;

impl Submitter {
    pub async fn login<It: IntoIterator<Item = Account>>(&mut self, accounts: It) -> Vec<Error> {
        let old_size = self.session.len();
        let mut err = Vec::new();
        join_all(accounts.into_iter().map(
            async move |Account {
                            handle,
                            password,
                            proxy,
                        }| {
                let mut p = Session::with_proxy(proxy).map_err(|e| Error {
                    operate: Operate::BuildClient,
                    kind: Kind::Judge(e),
                    handle: handle.clone(),
                })?;
                match p.login(handle, password.as_str()).await {
                    Ok(_) => Ok(p),
                    Err(e) => Err(Error {
                        operate: Operate::Login,
                        kind: Kind::Judge(e),
                        handle: p.handle,
                    }),
                }
            },
        ))
        .await
        .into_iter()
        .for_each(|x| match x {
            Ok(v) => self.session.push(v),
            Err(e) => err.push(e),
        });
        self.list.expand(self.session.len() - old_size);
        return err;
    }

    pub async fn add_session<It: IntoIterator<Item = Session>>(&mut self, sessions: It) {
        let p = sessions.into_iter();
        let old = self.session.len();
        self.session.extend(p);
        self.list.expand(self.session.len() - old);
    }

    pub async fn logout(&mut self) -> Vec<Error> {
        let ret: Vec<Error> =
            join_all(
                self.session
                    .iter_mut()
                    .map(async move |x| match x.logout().await {
                        Ok(_) => None,
                        Err(e) => Some(Error {
                            operate: Operate::Logout,
                            kind: Kind::Judge(e),
                            handle: x.handle.clone(),
                        }),
                    }),
            )
            .await
            .into_iter()
            .filter_map(|x| x)
            .collect();
        if ret.is_empty() {
            self.session.clear();
            self.list.clear();
        }
        ret
    }
}
