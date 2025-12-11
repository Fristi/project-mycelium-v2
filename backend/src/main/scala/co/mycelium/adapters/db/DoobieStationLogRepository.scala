package co.mycelium.adapters.db

import cats.tagless.{Derive, FunctorK}
import co.mycelium.domain.*
import co.mycelium.ports.StationLogRepository
import doobie.*
import doobie.implicits.*
import doobie.postgres.implicits.*

import java.time.Instant
import java.util.UUID
import scala.annotation.experimental

object DoobieStationLogRepository extends StationLogRepository[ConnectionIO] {
  override def insert(log: StationLog): ConnectionIO[Int] =
    sql"INSERT INTO station_log (station_id, occurred_on, event) VALUES (${log.stationId}, ${log.on}, ${log.event})".update.run

  override def listByStation(id: UUID, offset: Long): doobie.ConnectionIO[List[StationLog]] =
    sql"SELECT station_id, occurred_on, event FROM station_log WHERE station_id = $id ORDER BY occurred_on DESC LIMIT 30 OFFSET $offset"
      .query[StationLog]
      .to[List]

  override def lastTimeWatered(id: UUID): ConnectionIO[Option[Instant]] =
    sql"SELECT occurred_on FROM station_log WHERE station_id = $id AND event ->> '_type' = 'Watered' ORDER BY occurred_on DESC LIMIT 1"
      .query[Instant]
      .option
}
