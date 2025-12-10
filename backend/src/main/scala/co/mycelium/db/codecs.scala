package co.mycelium.db

import cats.*
import cats.data.*
import cats.implicits.*
import doobie.*
import doobie.implicits.*
import doobie.postgres.implicits.*
import io.circe.{Json, jawn}
import doobie.postgres.circe.json.implicits.*
import co.mycelium.domain.*
import cron4s.{Cron, CronExpr}
import io.circe.syntax.*

given Put[StationEvent] = Put[Json].contramap(_.asJson)
given Get[StationEvent] =
  Get[Json].temap(json => jawn.decode[StationEvent](json.noSpaces).leftMap(_.getMessage))

given Put[CronExpr] = Put[String].contramap(_.toString)
given Get[CronExpr] = Get[String].temap(s => Cron.parse(s).left.map(_.getMessage))
