#!/bin/sh

HOST=`host $KAFKA_BROKER | awk '/has address/ { print $4 ; exit }'`

## Wait until Kafka is ready then create demo topic
echo 'Waiting for Kafka to be ready...'
cub kafka-ready -b $HOST:9092 1 20 && \
sleep 1

echo "Creating Topic [$HOST:9092 <topic:'$KAFKA_TOPIC'>]"
kafka-topics                   \
    --create                   \
    --topic $KAFKA_TOPIC       \
    --if-not-exists            \
    --zookeeper zookeeper:2181 \
    --partitions 1             \
    --replication-factor 1
sleep 1

echo "Availalbe Topics"
kafka-topics --list --zookeeper zookeeper:2181
sleep 1

## Emit sample data stream
while true
    do echo "Sending Data [$HOST:9092 <topic:'$KAFKA_TOPIC'>]"
    for i in `seq 1 10`;
    do

        echo "$HOST"
        echo "$DATA"

        DATA="{\"data\":\"sample-data-$i\"}"

        echo "$DATA" | kafka-console-producer   \
            --broker-list $HOST:9092            \
            --topic $KAFKA_TOPIC     
    done
    sleep 1.0

    echo ''
    echo "Receiving Data [$HOST:9092 <topic:'$KAFKA_TOPIC'>]"

    kafka-console-consumer              \
        --bootstrap-server $HOST:9092   \
        --topic $KAFKA_TOPIC            \
        --from-beginning                \
        --max-messages 5

done
