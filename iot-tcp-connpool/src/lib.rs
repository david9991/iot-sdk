use futures::executor::block_on;
use iot_driver::{Config, Connection, ConnectionPool};
use std::{io, net::SocketAddr, str::FromStr};
use tokio_modbus::client::{
    tcp::{self},
    Context, Reader,
};

pub struct ModbusConnectionManager {
    socket_addr: SocketAddr,
}

impl bb8::ManageConnection for ModbusConnectionManager {
    type Connection = Context;
    type Error = io::Error;

    fn connect<'life0, 'async_trait>(
        &'life0 self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Self::Connection, Self::Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { tcp::connect(self.socket_addr).await })
    }

    fn is_valid<'life0, 'life1, 'async_trait>(
        &'life0 self,
        conn: &'life1 mut Self::Connection,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<(), Self::Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            match conn.read_coils(0, 1).await {
                Ok(_) => Ok(()),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Modbus connection broken",
                )),
            }
        })
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub struct ModbusConnection(bb8::PooledConnection<'static, ModbusConnectionManager>);
impl Connection for ModbusConnection {}
impl From<bb8::PooledConnection<'static, ModbusConnectionManager>> for ModbusConnection {
    fn from(value: bb8::PooledConnection<'static, ModbusConnectionManager>) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct ModbusConnectionPool {
    connection_pool: bb8::Pool<ModbusConnectionManager>,
}

impl<T: Config<Connection = ModbusConnection>> ConnectionPool<T> for ModbusConnectionPool {
    fn new(url: &str) -> Self {
        let socket_addr = SocketAddr::from_str(url).unwrap();
        let cm = ModbusConnectionManager { socket_addr };
        Self {
            connection_pool: block_on(bb8::Pool::builder().retry_connection(true).build(cm))
                .unwrap(),
        }
    }

    fn get(&self) -> impl std::future::Future<Output = <T as Config>::Connection> {
        async { ModbusConnection::from(self.connection_pool.get_owned().await.unwrap()) }
    }
}
