package co.mycelium.service

import blobstore.s3.S3Store
import cats.Monad
import cats.effect.Clock
import cats.effect.std.UUIDGen
import cats.implicits.*
import co.mycelium.db.Repositories
import co.mycelium.domain.*
import fs2.Stream

import java.util.UUID

trait StationService[F[_]] {
  def add(userId: String, station: StationInsert): F[UUID]
  def list(userId: String): F[List[Station]]
  def delete(userId: String, stationID: UUID): F[Unit]
  def update(userId: String, stationID: UUID, update: StationUpdate): F[Unit]
  def checkin(userId: String, stationID: UUID, measurements: List[StationMeasurement]): F[Watering]
  def details(
      userId: String,
      period: Option[MeasurementPeriod],
      stationID: UUID
  ): F[Either[Unit, StationDetails]]
  def watered(userId: String, stationID: UUID, watering: Watering): F[Unit]
  def getLogs(userId: String, stationID: UUID, page: Option[Long]): F[List[StationLog]]
  def classify(userId: String, stationID: UUID, image: Stream[F, Byte]): F[Unit]
}

final class StationServiceImpl[F[_]: Monad](
    uuidGen: UUIDGen[F],
    clock: Clock[F],
    repos: Repositories[F]
) extends StationService[F] {

  override def checkin(
      userId: String,
      stationID: UUID,
      measurements: List[StationMeasurement]
  ): F[Watering] =
    repos.stations.findById(stationID, userId).flatMap {
      case Some(station) =>
        repos.measurements.insertMany(stationID, measurements).as(Watering(None))
      case None =>
        Monad[F].pure(Watering(None))
    }

  override def add(userId: String, insert: StationInsert): F[UUID] =
    for {
      id      <- uuidGen.randomUUID
      created <- clock.realTimeInstant
      station = insert.toStation(id, created, userId)
      stationId <- repos.stations.insert(station, created)
    } yield stationId

  override def list(userId: String): F[List[Station]] =
    repos.stations.listByUserId(userId)

  override def delete(userId: String, stationID: UUID): F[Unit] =
    repos.stations.delete(stationID, userId).void

  override def update(userId: String, stationID: UUID, update: StationUpdate): F[Unit] =
    for {
      now <- clock.realTimeInstant
      _   <- repos.stations.update(stationID, userId, update, now)
    } yield ()

  override def details(
      userId: String,
      period: Option[MeasurementPeriod],
      stationID: UUID
  ): F[Either[Unit, StationDetails]] =
    repos.stations.findById(stationID, userId).flatMap {
      case Some(station) =>
        repos.measurements
          .avg(stationID, period.getOrElse(MeasurementPeriod.LastTwentyFourHours))
          .map(measurements => Right(StationDetails(station, measurements)))
      case None =>
        Monad[F].pure(Left(()))
    }

  override def watered(userId: String, stationID: UUID, watering: Watering): F[Unit] =
    for {
      now <- clock.realTimeInstant
      _   <- watering.watering match {
        case Some(value) =>
          repos.stationLog.insert(StationLog(stationID, now, StationEvent.Watered(value))).void
        case None => Monad[F].pure(())
      }
    } yield ()

  override def getLogs(userId: String, stationID: UUID, page: Option[Long]): F[List[StationLog]] =
    repos.stationLog.listByStation(stationID, page.getOrElse(0L))

  override def classify(userId: String, stationID: UUID, image: Stream[F, Byte]): F[Unit] =
    ???
}
