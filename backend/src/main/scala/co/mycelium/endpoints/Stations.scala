package co.mycelium.endpoints

import cats.effect.IO
import co.mycelium.CirceCodecs._
import co.mycelium.db.Repositories
import co.mycelium.domain._
import cron4s.CronExpr
import cron4s.lib.javatime.javaTemporalInstance
import org.http4s.HttpRoutes
import sttp.tapir._
import sttp.tapir.generic.Configuration
import sttp.tapir.generic.auto._
import sttp.tapir.json.circe._
import sttp.tapir.server.http4s.Http4sServerInterpreter

import java.time.Instant
import java.util.UUID
import scala.concurrent.duration.FiniteDuration

object Stations extends TapirSchemas {

  object endpoints {
    val stations = base.in("stations")

    val list = stations.get.out(jsonBody[List[Station]]).name("listStations")
    val add  = stations.post.in(jsonBody[StationInsert]).out(jsonBody[UUID]).name("addStation")
    val details = stations.get
      .in(path[UUID]("stationId"))
      .in(query[Option[MeasurementPeriod]]("period"))
      .name("getStation")
      .out(jsonBody[StationDetails])
    val update =
      stations.put.in(path[UUID]("stationId")).in(jsonBody[StationUpdate]).name("updateStation")
    val delete = stations.delete.in(path[UUID]("stationId")).name("deleteStation")
    val checkIn = stations
      .in(path[UUID]("stationId"))
      .in("checkin")
      .put
      .in(jsonBody[List[StationMeasurement]])
      .name("checkinStation")
      .out(jsonBody[Watering])
    val watered = stations
      .in(path[UUID]("stationId"))
      .in("watered")
      .post
      .in(jsonBody[Watering])
      .name("wateredAtStation")
    val log = stations
      .in(path[UUID]("stationId"))
      .in("log")
      .in(query[Option[Long]]("page"))
      .name("getStationLog")
      .out(jsonBody[List[StationLog]])

    val all = Set(list, add, details, update, delete, checkIn, watered, log)
  }

  def routes(repos: Repositories[IO]): HttpRoutes[IO] = {

    val list =
      endpoints.list.serverLogic(at => _ => repos.stations.listByUserId(at.sub).map(Right(_)))

    val add = endpoints.add.serverLogic { at => insert =>
      val id      = UUID.randomUUID()
      val created = Instant.now()
      val station = insert.toStation(id, created, at.sub)

      repos.stations.insert(station, created).map(Right(_))
    }

    val delete =
      endpoints.delete.serverLogic(at => id => repos.stations.delete(id, at.sub).as(Right(())))

    val checkin = endpoints.checkIn.serverLogic { at =>
      { case (id, measurements) =>
        for {
          stationOpt <- repos.stations.findById(id, at.sub)
          _          <- repos.measurements.insertMany(id, measurements)
        } yield Right(Watering(None))
      }
    }

    val watered = endpoints.watered.serverLogic { at =>
      { case (id, request) =>
        request.watering match {
          case Some(watered) =>
            repos.stationLog
              .insert(StationLog(id, Instant.now(), StationEvent.Watered(watered)))
              .as(Right(()))
          case None => IO.unit.as(Right(()))
        }
      }
    }

    val log = endpoints.log.serverLogic { at =>
      { case (id, page) =>
        repos.stationLog.listByStation(id, page.getOrElse(0L) * 30).map(Right(_))
      }
    }

    val details = endpoints.details.serverLogic { at =>
      { case (id, period) =>
        repos.stations.findById(id, at.sub).flatMap {
          case Some(station) =>
            repos.measurements
              .avg(id, period.getOrElse(MeasurementPeriod.LastTwentyFourHours))
              .map(measurements => Right(StationDetails(station, measurements)))
          case None =>
            IO.delay(Left(()))
        }
      }
    }

    val update = endpoints.update.serverLogic { at =>
      { case (id, update) =>
        repos.stations.update(id, at.sub, update, Instant.now()).as(Right(()))
      }
    }

    Http4sServerInterpreter[IO]().toRoutes(
      List(list, add, delete, log, watered, checkin, details, update)
    )
  }
}

trait TapirSchemas {
  implicit val customConfiguration: Configuration =
    Configuration.default.withDiscriminator("_type")

  implicit val schemaCronExpr: Schema[CronExpr]             = Schema.string
  implicit val schemaFiniteDuration: Schema[FiniteDuration] = Schema.string

  implicit val codecMeasurementPeriod: Codec[String, MeasurementPeriod, CodecFormat.TextPlain] =
    Codec.string.map(
      Mapping.fromDecode((str: String) =>
        DecodeResult.fromOption(MeasurementPeriod.fromString(str))
      )(_.repr)
    )
}
