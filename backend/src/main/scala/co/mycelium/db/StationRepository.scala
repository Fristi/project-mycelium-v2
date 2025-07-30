package co.mycelium.db

import cats.{Applicative, Traverse}
import cats.data.NonEmptyList
import cats.tagless.{Derive, FunctorK}
import cats.implicits._
import co.mycelium.domain._
import doobie._
import doobie.implicits._
import doobie.postgres.implicits._

import java.time.Instant
import java.util.UUID

trait StationRepository[F[_]] {
  def insert(station: Station, on: Instant): F[UUID]
  def listByUserId(userId: String): F[List[Station]]
  def findById(id: UUID, userId: String): F[Option[Station]]
  def delete(id: UUID, userId: String): F[Int]
  def update(id: UUID, userId: String, update: StationUpdate, now: Instant): F[Int]
}

object StationRepository {
  implicit val functorK: FunctorK[StationRepository] = Derive.functorK
}

object DoobieStationRepository extends StationRepository[ConnectionIO] {
  def insert(station: Station, on: Instant): ConnectionIO[UUID] = {

    def insertIntoStations =
      sql"""INSERT INTO stations (id, mac_addr, name, location, description, user_id, created) 
            VALUES (${station.id}, ${station.mac}, ${station.name}, ${station.location}, ${station.description}, ${station.userId}, $on) 
            on conflict on constraint unique_mac 
            do update set 
            updated = now(), 
            user_id = excluded.user_id 
            returning id
      """.query[UUID]

    insertIntoStations.unique
  }

  def listByUserId(userId: String): ConnectionIO[List[Station]] =
    sql"SELECT id, mac_addr, name, location, description, user_id, created, updated FROM stations where user_id = $userId"
      .query[Station]
      .to[List]

  def findById(id: UUID, userId: String): ConnectionIO[Option[Station]] =
    sql"SELECT id, mac_addr, name, location, description, user_id, created, updated FROM stations WHERE id = $id AND user_id = $userId"
      .query[Station]
      .option

  def delete(id: UUID, userId: String): ConnectionIO[Int] =
    sql"DELETE FROM stations WHERE id = $id AND user_id = $userId".update.run

  override def update(
      id: UUID,
      userId: String,
      update: StationUpdate,
      now: Instant
  ): ConnectionIO[Int] = {

    val updateAttributes = {
      val updates = List(
        update.name.map(n => fr"name = $n"),
        update.location.map(n => fr"location = $n"),
        update.description.map(n => fr"description = $n")
      )

      NonEmptyList.fromList(updates.flatten) match {
        case Some(update) =>
          fr"UPDATE stations ${Fragments.set(update :+ fr"updated = $now")} WHERE id = $id AND user_id = $userId".update.run
        case None =>
          Applicative[ConnectionIO].pure(0)
      }
    }

    updateAttributes
  }
}
