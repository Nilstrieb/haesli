use crate::Result;
use amqp_core::{
    amqp_todo,
    connection::Channel,
    methods::{Method, QueueBind, QueueDeclare, QueueDeclareOk},
    queue::{QueueDeletion, QueueId, QueueName, RawQueue},
    GlobalData,
};
use parking_lot::Mutex;
use std::sync::{atomic::AtomicUsize, Arc};

pub fn declare(channel: Channel, queue_declare: QueueDeclare) -> Result<Method> {
    let QueueDeclare {
        queue: queue_name,
        passive,
        durable,
        exclusive,
        auto_delete,
        no_wait,
        arguments,
        ..
    } = queue_declare;

    let queue_name = QueueName::new(queue_name.into());

    if !arguments.is_empty() {
        amqp_todo!();
    }

    // todo: durable is technically spec-compliant, the spec doesn't really require it, but it's a todo
    // not checked here because it's the default for amqplib which is annoying
    if passive || no_wait {
        amqp_todo!();
    }

    let global_data = {
        let global_data = channel.global_data.clone();

        let id = QueueId::random();
        let queue = Arc::new(RawQueue {
            id,
            name: queue_name.clone(),
            messages: Mutex::default(),
            durable,
            exclusive: exclusive.then(|| channel.id),
            deletion: if auto_delete {
                QueueDeletion::Auto(AtomicUsize::default())
            } else {
                QueueDeletion::Manual
            },
            consumers: Mutex::default(),
        });

        {
            let mut global_data_lock = global_data.lock();
            global_data_lock.queues.insert(queue_name.clone(), queue);
        }

        global_data
    };

    bind_queue(global_data, (), queue_name.clone().into_inner())?;

    Ok(Method::QueueDeclareOk(QueueDeclareOk {
        queue: queue_name.to_string(),
        message_count: 0,
        consumer_count: 0,
    }))
}

pub async fn bind(_channel_handle: Channel, _queue_bind: QueueBind) -> Result<Method> {
    amqp_todo!();
}

fn bind_queue(global_data: GlobalData, _exchange: (), routing_key: Arc<str>) -> Result<()> {
    let mut global_data = global_data.lock();

    // todo: don't
    let queue = global_data
        .queues
        .get(&QueueName::new(routing_key.clone()))
        .unwrap()
        .clone();
    global_data
        .default_exchange
        .insert(routing_key.to_string(), queue);

    Ok(())
}
