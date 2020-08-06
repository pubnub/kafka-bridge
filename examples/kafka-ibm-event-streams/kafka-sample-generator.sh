#!/bin/sh

echo "Creating Topic [$KAFKA_BROKER <topic:'$KAFKA_TOPIC'>]"
kafka-topics                     \
    --create                     \
    --topic $KAFKA_TOPIC         \
    --partitions 3               \
    --replication-factor 3       \
    --if-not-exists              \
    --zookeeper $KAFKA_ZOOKEEPER
sleep 1

echo "Availalbe Topics"
kafka-topics --list --zookeeper $KAFKA_ZOOKEEPER
sleep 1

## Emit sample data stream
while true
    do echo "Sending Data [$KAFKA_BROKER <topic:'$KAFKA_TOPIC'>]"
    for i in `seq 1 10`;
    do

        echo "$KAFKA_BROKER"
        echo "$DATA"

        DATA="{\"data\":\"sample-data-$i\"}"

        echo "$DATA" | kafka-console-producer                                  \
            --broker-list $KAFKA_BROKER                                        \
            --topic $KAFKA_TOPIC                                               \
            --producer.config /etc/kafka/secrets/host.producer.sasl.config
    done
    sleep 1.0

    echo ''
    echo "Receiving Data [$KAFKA_BROKER <topic:'$KAFKA_TOPIC'>]"

    kafka-console-consumer                                                 \
        --bootstrap-server $KAFKA_BROKER                                   \
        --topic $KAFKA_TOPIC                                               \
        --from-beginning                                                   \
        --max-messages 5                                                   \
        --consumer.config /etc/kafka/secrets/host.consumer.sasl.config
done
