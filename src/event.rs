use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::*;


#[derive(Clone, Default)]
pub struct Receiver<T:Clone> {
    event_queue: RefCell<Vec<T>>
}

impl<T:Clone> Receiver<T> {
    pub fn new() -> Rc<Receiver<T>> {
        Rc::new(Self{
            event_queue: RefCell::new(Vec::new())
        })
    }
    pub fn receive(&self, event: T) {
        self.event_queue.borrow_mut().push(event)
    }
    pub fn poll(&self) -> Vec<T> {
        self.event_queue.borrow_mut().drain(..).collect::<Vec<T>>()
    }
}


#[derive(Clone)]
pub struct Locked;
#[derive(Clone)]
pub struct Unlocked;

#[derive(Clone, Default)]
pub struct Sender<T:Clone, State=Unlocked> {
    pub receivers: Rc<RefCell<Vec<Rc<Receiver<T>>>>>,
    pub routed: Rc<RefCell<HashMap<Uuid, Rc<Receiver<T>>>>>,
    state: std::marker::PhantomData<State>
}

impl<T:Clone,State> Sender<T, State> {
    pub fn new() -> Self {
        Self {
            receivers: Rc::new(RefCell::new(Vec::new())),
            routed: Rc::new(RefCell::new(HashMap::new())),
            state: std::marker::PhantomData
        }
    }
    pub fn new_receiver(&self) -> Rc<Receiver<T>> {
        let r = Receiver::new();
        self.receivers.borrow_mut().push(r.clone());
        r
    }
    pub fn new_routed_receiver(&self, uuid: Option<Uuid>) -> (Uuid, Rc<Receiver<T>>) {
        let r = Receiver::new();
        let uuid = if let Some(u) = uuid {
            u
        } else {
            Uuid::new_v4()
        };
        self.routed.borrow_mut().insert(uuid, r.clone());
        (uuid,r)
    }
}

impl<T:Clone> Sender<T, Unlocked> {
    pub fn send(&self, event: T) {
        for r in self.receivers.borrow().iter() {
            r.receive(event.clone())
        }
    }
    pub fn route(&self, uuid: Uuid, event: T) {
        if let Some(r) = self.routed.borrow().get(&uuid) {
            r.receive(event);
        }
    }
    pub fn lock(&self) -> Sender<T,Locked> {
        Sender {
            receivers: self.receivers.clone(),
            routed: self.routed.clone(),
            state: std::marker::PhantomData
        }
    }
}