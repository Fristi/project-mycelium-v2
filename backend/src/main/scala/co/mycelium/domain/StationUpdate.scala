package co.mycelium.domain

final case class StationUpdate(
    name: Option[String] = None,
    location: Option[String] = None,
    description: Option[String] = None
)
