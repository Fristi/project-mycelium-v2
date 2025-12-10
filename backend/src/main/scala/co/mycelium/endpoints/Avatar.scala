package co.mycelium.endpoints

import cats.effect.IO
import fs2.*
import org.http4s.dsl.io.*
import org.http4s.{HttpRoutes, Response}

object Avatar {

  def routes = HttpRoutes.of[IO] {
    case GET -> Root / UUIDVar(uuid) =>
      val stream      = getClass.getResourceAsStream("/placeholder.png")
      val placeholder = io.readInputStream[IO](IO.delay(stream), 1024)
      val payload     = placeholder

      IO.delay(Response(body = payload))

    case req @ PUT -> Root / UUIDVar(uuid) =>
      ???
  }
}
