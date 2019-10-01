#!/bin/sh

KAFKA_BROKER=kafka

## Wait until Kafka is ready then create demo topic
echo 'Waiting for Kafka to be ready...'
#kafka-ready.sh -b $KAFKA_BROKER:9092 1 20
sleep 10
echo "Creating Topic [$KAFKA_TOPIC]"
kafka-topics.sh --create --if-not-exists --zookeeper zookeeper:2181 --partitions 1 --replication-factor 1 --topic $KAFKA_TOPIC
sleep 2
echo "Availalbe Topics"
echo `kafka-topics.sh --list --zookeeper zookeeper:2181`

## Emmit sample data once per second
while true
    do echo "Sending sample data [$KAFKA_TOPIC]"
    echo `kafka-console-producer.sh --broker-list $KAFKA_BROKER:9092 --topic $KAFKA_TOPIC '{"data":"sample-data"}'`
    sleep 1
    echo ''
    echo 'Receiving data from [$KAFKA_TOPIC]'
    #echo `kafka-console-consumer.sh --bootstrap-server $KAFKA_BROKER:9092 --topic $KAFKA_TOPIC --partition 0 --from-beginning --timeout-ms 5000 --skip-message-on-error`
    kafka-simple-consumer-shell.sh --broker-list $KAFKA_BROKER:9092 --topic test --partition 0
done
