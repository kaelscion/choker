use std::task::{Context, Poll};
use std::pin::Pin;
use hyper::client::conn::http2::Connection;
use tokio::net::TcpStream;
use tokio::io::{ReadBuf, AsyncWrite, AsyncRead};
use async_speed_limit::{limiter::{Limiter, Consume}, clock::StandardClock};
use std::io::Error;
use hyper::upgrade::Upgraded;

use pin_project::pin_project;

#[pin_project]
pub struct LimitedUpgraded {
    #[pin]
    io: Upgraded,
    lim: Limiter,
    lim_fut: Option<Consume<StandardClock, ()>>
}

impl LimitedUpgraded {

    pub async fn from_upgraded(upgraded: Upgraded, limiter: Limiter) -> Result<LimitedUpgraded, Error> {
        let this  = LimitedUpgraded{
            io: upgraded,
            lim: limiter,
            lim_fut: None,
        };  
        Ok(this)
    }
}


impl AsyncWrite for LimitedUpgraded {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        let this = self.project();
        if let Some(fut) = this.lim_fut {
            if Pin::new(fut).poll(cx).is_pending() {
                return Poll::Pending
            }

            *this.lim_fut = None;
        }

        if let Poll::Ready(res) = this.io.poll_write(cx, buf) {
            match res {
                Ok(sz) => {
                    if sz > 0 {
                        *this.lim_fut = Some(this.lim.consume(sz))
                    }
                    Poll::Ready(Ok(sz))
                },
                Err(e) => Poll::Ready(Err(e))
            }
        }else{
            Poll::Pending
        }
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<Result<usize, Error>> {
        let this = self.project();
        this.io.poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let this = self.project();
        this.io.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let this = self.project();
        this.io.poll_shutdown(cx)
    }
}

impl AsyncRead for LimitedUpgraded {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>,) -> Poll<Result<(), Error>> {
        let io:Pin<&mut Upgraded> = Pin::new(&mut self.get_mut().io);
        io.poll_read(cx, buf)
    }
}


#[pin_project]
pub struct LimitedStream {
    #[pin]
    io: TcpStream,
    lim: Limiter,
    lim_fut: Option<Consume<StandardClock, ()>>
}

impl LimitedStream {

    pub async fn connect(addr: &str, lim: Limiter) -> Result<LimitedStream, Error> {
        println!("connecting ... {}", addr); 
        let io = TcpStream::connect(addr).await?;
        println!("connected");
        Ok(LimitedStream{
            io,
            lim,
            lim_fut: None,
        })
    }
}

impl AsyncWrite for LimitedStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        let this = self.project();
        if let Some(fut) = this.lim_fut {
            if Pin::new(fut).poll(cx).is_pending() {
                return Poll::Pending
            }

            *this.lim_fut = None;
        }

        if let Poll::Ready(res) = this.io.poll_write(cx, buf) {
            match res {
                Ok(sz) => {
                    if sz > 0 {
                        *this.lim_fut = Some(this.lim.consume(sz))
                    }
                    Poll::Ready(Ok(sz))
                },
                Err(e) => Poll::Ready(Err(e))
            }
        }else{
            Poll::Pending
        }
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<Result<usize, Error>> {
        let this = self.project();
        this.io.poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let this = self.project();
        this.io.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let this = self.project();
        this.io.poll_shutdown(cx)
    }
}

impl AsyncRead for LimitedStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>,) -> Poll<Result<(), Error>> {
        let io:Pin<&mut TcpStream> = Pin::new(&mut self.get_mut().io);
        io.poll_read(cx, buf)
    }
}

impl Connection for LimitedStream {
    fn connected(&self) -> Connected {
        let connected = Connected::new();
        connected
    }
}
impl Connection for LimitedUpgraded {
    fn connected(&self) -> Connected {
        let connected = Connected::new();
        connected
    }
}