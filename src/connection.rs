use crate::*;

#[derive(Copy, Clone)]
pub struct ObjectResult<'a> {
    pub path: &'a str,
    pub id: u32,
    pub ty: u32,
}
impl core::fmt::Debug for ObjectResult<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{} @0x{:08x} type={:08x}", self.path, self.id, self.ty)
    }
}

pub struct SignatureResult<'a> {
    pub object: ObjectResult<'a>,
    pub name: &'a str,
    pub args: &'a mut dyn Iterator<Item = (&'a str, BlobMsgType)>,
}

pub struct Connection<T: IO> {
    io: T,
    peer: u32,
    sequence: u16,
    buffer: [u8; 64 * 1024],
}

impl<T: IO> Connection<T> {
    /// Create a new ubus connection from an existing IO
    pub fn new(io: T) -> Result<Self, Error<T::Error>> {
        let mut new = Self {
            io,
            peer: 0,
            sequence: 0,
            buffer: [0u8; 64 * 1024],
        };

        // ubus server should say hello on connect
        let message = new.next_message()?;

        // Verify the header is what we expect
        valid_data!(
            message.header.message == MessageType::HELLO,
            "Expected hello"
        );

        // Record our peer id
        new.peer = message.header.peer.into();

        Ok(new)
    }

    // Get next message from ubus channel (blocking!)
    pub fn next_message(&mut self) -> Result<Message, Error<T::Error>> {
        Message::from_io(&mut self.io, &mut self.buffer)
    }

    pub fn send(&mut self, message: MessageBuilder) -> Result<(), Error<T::Error>> {
        self.io.put(message.into())
    }

    pub fn invoke(
        &mut self,
        obj: u32,
        method: &str,
        args: &[BlobMsgData],
        mut on_result: impl FnMut(BlobIter<BlobMsg>),
    ) -> Result<(), Error<T::Error>> {
        self.sequence += 1;
        let sequence = self.sequence.into();

        let mut buffer = [0u8; 1024];
        let mut message = MessageBuilder::new(
            &mut buffer,
            MessageHeader {
                version: MessageVersion::CURRENT,
                message: MessageType::INVOKE,
                sequence,
                peer: obj.into(),
            },
        )
        .unwrap();

        message.put(MessageAttr::ObjId(obj))?;
        message.put(MessageAttr::Method(method))?;
        message.put(MessageAttr::Data(&[]))?;

        self.send(message)?;
        'message: loop {
            let message = self.next_message()?;
            if message.header.sequence != sequence {
                continue;
            }

            let attrs = BlobIter::<MessageAttr>::new(message.blob.data);

            match message.header.message {
                MessageType::STATUS => {
                    for attr in attrs {
                        if let MessageAttr::Status(0) = attr {
                            return Ok(());
                        } else if let MessageAttr::Status(status) = attr {
                            return Err(Error::Status(status));
                        }
                    }
                    return Err(Error::InvalidData("Invalid status message"));
                }
                MessageType::DATA => {
                    for attr in attrs {
                        if let MessageAttr::Data(data) = attr {
                            on_result(BlobIter::<BlobMsg>::new(data));
                            continue 'message;
                        }
                    }
                    return Err(Error::InvalidData("Invalid data message"));
                }
                unknown => {
                    std::dbg!(unknown);
                }
            }
        }
    }

    pub fn lookup(
        &mut self,
        mut on_object: impl FnMut(ObjectResult),
        mut on_signature: impl FnMut(SignatureResult),
    ) -> Result<(), Error<T::Error>> {
        self.sequence += 1;
        let sequence = self.sequence.into();

        let mut buffer = [0u8; 1024];
        let message = MessageBuilder::new(
            &mut buffer,
            MessageHeader {
                version: MessageVersion::CURRENT,
                message: MessageType::LOOKUP,
                sequence,
                peer: 0.into(),
            },
        )
        .unwrap();

        self.send(message)?;

        loop {
            let message = self.next_message()?;
            if message.header.sequence != sequence {
                continue;
            }

            let attrs = BlobIter::<MessageAttr>::new(message.blob.data);

            if message.header.message == MessageType::STATUS {
                for attr in attrs {
                    if let MessageAttr::Status(0) = attr {
                        return Ok(());
                    } else if let MessageAttr::Status(status) = attr {
                        return Err(Error::Status(status));
                    }
                }
                return Err(Error::InvalidData("Invalid status message"));
            }

            if message.header.message != MessageType::DATA {
                continue;
            }

            let mut obj_path: Option<&str> = None;
            let mut obj_id: Option<u32> = None;
            let mut obj_type: Option<u32> = None;
            for attr in attrs {
                match attr {
                    MessageAttr::ObjPath(path) => obj_path = Some(path),
                    MessageAttr::ObjId(id) => obj_id = Some(id),
                    MessageAttr::ObjType(ty) => obj_type = Some(ty),
                    MessageAttr::Signature(nested) => {
                        let object = ObjectResult {
                            path: obj_path.unwrap(),
                            id: obj_id.unwrap(),
                            ty: obj_type.unwrap(),
                        };
                        on_object(object);

                        for signature in nested {
                            if let BlobMsgData::Table(table) = signature.data {
                                on_signature(SignatureResult {
                                    object,
                                    name: signature.name.unwrap(),
                                    args: &mut table.map(|arg| {
                                        if let BlobMsgData::Int32(typeid) = arg.data {
                                            (arg.name.unwrap(), BlobMsgType::from(typeid as u32))
                                        } else {
                                            panic!()
                                        }
                                    }),
                                });
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
    }
}
