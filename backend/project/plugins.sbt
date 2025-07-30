addSbtPlugin("org.scalameta" % "sbt-scalafmt" % "2.5.2")

addSbtPlugin("nl.gn0s1s" % "sbt-dotenv" % "3.1.1")

addSbtPlugin("de.gccc.sbt" % "sbt-jib" % "1.4.2")

addSbtPlugin("org.scalameta" % "sbt-metals" % "1.6.0")

libraryDependencies += "com.google.cloud.tools" % "jib-core" % "0.27.3"