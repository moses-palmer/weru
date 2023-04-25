use super::*;

use futures::StreamExt;

use crate::traits::Channel;

macro_rules! engine {
    () => {
        if let Some(configuration) = configuration() {
            configuration.engine().await.unwrap()
        } else {
            return;
        }
    };
}

/// Writes to a channel.
///
/// # Arguments
/// *  `channel` The channel to which to write.
/// *  `events` - The events to write.
///
/// # Panics
/// This function will panic on any error.
async fn write<T>(channel: &dyn Channel<T>, events: &[T])
where
    T: Event,
{
    for event in events {
        channel.broadcast(event.clone()).await.unwrap();
    }
}

/// Reads a number of events from a listener.
///
/// # Arguments
/// *  `listener` - The listener from which to read.
/// *  `count` - The number of events to read.
///
/// # Panics
/// This function will panic on any error.
async fn read<T>(
    listener: BoxStream<'static, Result<T, Error>>,
    count: usize,
) -> Vec<T>
where
    T: Event,
{
    listener.take(count).map(Result::unwrap).collect().await
}

#[actix_rt::test]
async fn listen_receive_spsc() {
    // Arrange
    let topic = "listen_receive_spsc".to_string();
    let engine = engine!();
    let sender = engine.channel(topic.clone()).await.unwrap();
    let channel = engine.channel::<String>(topic.clone()).await.unwrap();
    let events = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let expected = events.clone();

    // Act
    let listener = channel.listen().await.unwrap();
    write(sender.as_ref(), &events).await;
    let actual = read(listener, events.len()).await;

    // Assert
    assert_eq!(expected, actual);
}

#[actix_rt::test]
async fn listen_receive_spmc() {
    // Arrange
    let topic = "listen_receive_spmc".to_string();
    let engine = engine!();
    let sender = engine.channel(topic.clone()).await.unwrap();
    let channel = engine.channel::<String>(topic.clone()).await.unwrap();
    let events = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let expected = events.clone();

    // Act
    let listener1 = channel.listen().await.unwrap();
    let listener2 = channel.listen().await.unwrap();
    write(sender.as_ref(), &events).await;
    let actual1 = read(listener1, expected.len()).await;
    let actual2 = read(listener2, expected.len()).await;

    // Assert
    assert_eq!(expected, actual1);
    assert_eq!(expected, actual2);
}

#[actix_rt::test]
async fn listen_receive_mpsc() {
    // Arrange
    let topic = "listen_receive_mpsc".to_string();
    let engine = engine!();
    let sender1 = engine.channel(topic.clone()).await.unwrap();
    let sender2 = engine.channel(topic.clone()).await.unwrap();
    let channel = engine.channel::<String>(topic.clone()).await.unwrap();
    let events1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let events2 = vec!["d".to_string(), "e".to_string(), "f".to_string()];
    let expected = {
        let mut expected = events1.clone();
        expected.extend(events2.clone());
        expected
    };

    // Act
    let listener = channel.listen().await.unwrap();
    write(sender1.as_ref(), &events1).await;
    write(sender2.as_ref(), &events2).await;
    let actual = read(listener, expected.len()).await;

    // Assert
    assert_eq!(expected, actual);
}

#[actix_rt::test]
async fn listen_receive_mpmc() {
    // Arrange
    let topic = "listen_receive_mpmc".to_string();
    let engine = engine!();
    let sender1 = engine.channel(topic.clone()).await.unwrap();
    let sender2 = engine.channel(topic.clone()).await.unwrap();
    let channel1 = engine.channel::<String>(topic.clone()).await.unwrap();
    let channel2 = engine.channel::<String>(topic.clone()).await.unwrap();
    let events1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let events2 = vec!["d".to_string(), "e".to_string(), "f".to_string()];
    let expected = {
        let mut expected = events1.clone();
        expected.extend(events2.clone());
        expected
    };

    // Act
    let listener1 = channel1.listen().await.unwrap();
    let listener2 = channel2.listen().await.unwrap();
    write(sender1.as_ref(), &events1).await;
    write(sender2.as_ref(), &events2).await;
    let actual1 = read(listener1, expected.len()).await;
    let actual2 = read(listener2, expected.len()).await;

    // Assert
    assert_eq!(expected, actual1);
    assert_eq!(expected, actual2);
}
