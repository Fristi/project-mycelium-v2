package co.mycelium.domain

import scala.concurrent.duration.FiniteDuration

sealed trait StationEvent

object StationEvent {

  case class Watered(period: FiniteDuration) extends StationEvent

}
