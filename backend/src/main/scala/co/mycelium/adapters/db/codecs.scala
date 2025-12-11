package co.mycelium.adapters.db

import cats.*
import cats.data.*
import cats.implicits.*
import doobie.*
import doobie.implicits.*
import doobie.postgres.implicits.*
import io.circe.{Json, jawn}
import doobie.postgres.circe.json.implicits.*
import co.mycelium.domain.*
import io.circe.syntax.*

given Put[StationEvent] = Put[Json].contramap(_.asJson)
given Get[StationEvent] =
  Get[Json].temap(json => jawn.decode[StationEvent](json.noSpaces).leftMap(_.getMessage))
