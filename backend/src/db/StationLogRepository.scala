package co.mycelium.db

import cats.tagless.{Derive, FunctorK}
import co.mycelium.domain.*
import doobie.*
import doobie.implicits.*
import doobie.postgres.implicits.*
import doobie.postgres.circe.json.implicits.*

import java.time.Instant
import java.util.UUID
import scala.annotation.experimental

trait StationLogRepository[F[_]] {
  def insert(log: StationLog): F[Int]
  def listByStation(id: UUID, offset: Long): F[List[StationLog]]
  def lastTimeWatered(id: UUID): F[Option[Instant]]
}

object StationLogRepository {
  implicit val functorK: FunctorK[StationLogRepository] = Derive.functorK
}

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
