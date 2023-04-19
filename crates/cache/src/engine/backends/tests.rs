use std::thread::sleep;

use super::*;

macro_rules! engine {
    () => {
        if let Some(configuration) = configuration() {
            configuration.engine().await.unwrap()
        } else {
            return;
        }
    };
}

#[actix_rt::test]
async fn get_none() {
    // Arrange
    let name = "get_none".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "unknown".to_string();
    let expected = None;

    // Act
    let actual = cache.get(&key).await;

    // Assert
    assert_eq!(Ok(expected), actual);
}

#[actix_rt::test]
async fn put_get() {
    // Arrange
    let name = "put_get".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "key".to_string();
    let expected = Some("expected".to_string());

    // Act
    cache
        .put(
            key.clone(),
            expected.clone().unwrap(),
            Duration::from_secs(32),
        )
        .await
        .unwrap();
    let actual = cache.get(&key).await;

    // Assert
    assert_eq!(Ok(expected), actual);
}

#[actix_rt::test]
async fn put_get_multiple() {
    // Arrange
    let name = "put_get".to_string();
    let engine = engine!();
    let cache1 = engine.cache::<String, String>(&name).await.unwrap();
    let cache2 = engine.cache::<String, String>(&name).await.unwrap();
    let key = "key".to_string();
    let expected = Some("expected".to_string());

    // Act
    cache1
        .put(
            key.clone(),
            expected.clone().unwrap(),
            Duration::from_secs(32),
        )
        .await
        .unwrap();
    let actual = cache2.get(&key).await;

    // Assert
    assert_eq!(Ok(expected), actual);
}

#[actix_rt::test]
async fn put_get_expired() {
    // Arrange
    let name = "put_get_expired".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "key".to_string();
    let expected = None;

    // Act
    cache
        .put(key.clone(), "value".to_string(), Duration::from_secs(1))
        .await
        .unwrap();
    sleep(Duration::from_millis(1500));
    let actual = cache.get(&key).await;

    // Assert
    assert_eq!(Ok(expected), actual);
}

#[actix_rt::test]
async fn pop_none() {
    // Arrange
    let name = "pop_none".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "unknown".to_string();
    let expected = None;

    // Act
    let actual = cache.pop(&key).await;

    // Assert
    assert_eq!(Ok(expected), actual);
}

#[actix_rt::test]
async fn put_pop() {
    // Arrange
    let name = "put_pop".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "key".to_string();
    let expected = Some("expected".to_string());

    // Act
    cache
        .put(
            key.clone(),
            expected.clone().unwrap(),
            Duration::from_secs(32),
        )
        .await
        .unwrap();
    let actual = cache.pop(&key).await;

    // Assert
    assert_eq!(Ok(expected), actual);
    assert_eq!(Ok(None), cache.get(&key).await);
}

#[actix_rt::test]
async fn put_pop_expired() {
    // Arrange
    let name = "put_pop_expired".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "key".to_string();
    let expected = None;

    // Act
    cache
        .put(key.clone(), "value".to_string(), Duration::from_secs(1))
        .await
        .unwrap();
    sleep(Duration::from_millis(1500));
    let actual = cache.pop(&key).await;

    // Assert
    assert_eq!(Ok(expected), actual);
}

#[actix_rt::test]
async fn replace_none() {
    // Arrange
    let name = "replace_none".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "unknown".to_string();
    let expected = None;

    // Act
    let old = cache.replace(key.clone(), "value".to_string(), None).await;
    let actual = cache.get(&key).await;

    // Assert
    assert_eq!(Ok(None), old);
    assert_eq!(Ok(expected), actual);
}

#[actix_rt::test]
async fn put_replace() {
    // Arrange
    let name = "put_replace".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "unknown".to_string();
    let expected1 = Some("expected".to_string());
    let expected2 = Some("expected2".to_string());

    // Act
    cache
        .put(
            key.clone(),
            expected1.clone().unwrap(),
            Duration::from_secs(32),
        )
        .await
        .unwrap();
    let actual1 = cache
        .replace(key.clone(), expected2.clone().unwrap(), None)
        .await;
    let actual2 = cache.get(&key).await;

    // Assert
    assert_eq!(Ok(expected1), actual1);
    assert_eq!(Ok(expected2), actual2);
}

#[actix_rt::test]
async fn put_replace_expired() {
    // Arrange
    let name = "put_replace_expired".to_string();
    let engine = engine!();
    let cache = engine.cache::<String, String>(&name).await.unwrap();
    let key = "unknown".to_string();
    let expected1 = None;
    let expected2 = None;

    // Act
    cache
        .put(key.clone(), "value1".to_string(), Duration::from_secs(1))
        .await
        .unwrap();
    sleep(Duration::from_millis(1500));
    let actual1 = cache.replace(key.clone(), "value2".to_string(), None).await;
    let actual2 = cache.get(&key).await;

    // Assert
    assert_eq!(Ok(expected1), actual1);
    assert_eq!(Ok(expected2), actual2);
}
