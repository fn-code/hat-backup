use std::mem;
use std::ops;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};


pub struct SyncPool<V> {
    vals: Mutex<Vec<V>>,
    cond: Condvar,
}

pub struct SyncPoolGuard<'mutex, V: 'mutex> {
    m: &'mutex SyncPool<V>,
    v: Option<V>,
}

impl<'mutex, V> ops::Deref for SyncPoolGuard<'mutex, V> {
    type Target = V;
    fn deref(&self) -> &V {
        self.v.as_ref().unwrap()
    }
}

impl<'mutex, V> ops::DerefMut for SyncPoolGuard<'mutex, V> {
    fn deref_mut(&mut self) -> &mut V {
        self.v.as_mut().unwrap()
    }
}

impl<'mutex, V> ops::Drop for SyncPoolGuard<'mutex, V> {
    fn drop(&mut self) {
        let mut vals = self.m.vals.lock().unwrap();
        let v = mem::replace(&mut self.v, None);
        vals.push(v.unwrap());
        self.m.cond.notify_one();
    }
}

impl<V> SyncPool<V> {
    pub fn new(vals: Vec<V>) -> SyncPool<V> {
        SyncPool {
            vals: Mutex::new(vals),
            cond: Condvar::new(),
        }
    }

    pub fn lock(&self) -> Result<SyncPoolGuard<V>, PoisonError<MutexGuard<Vec<V>>>> {
        let v = {
            let mut vs = try!(self.vals.lock());
            while vs.len() == 0 {
                vs = try!(self.cond.wait(vs));
            }
            vs.pop().unwrap()
        };
        Ok(SyncPoolGuard {
            m: &self,
            v: Some(v),
        })
    }
}