package com.dropbear.sample

DynamicBoundingBox {
    Circle {
        name = "status_backing"

        align {
            horizontal = Alignment.Left + 10
            vertical = Alignment.Centered
        }

        radius = 40.0

        Image {
            name = "status_image"

            when (Player.playerState) {
                PlayerState.Gas -> {
                    resource = "euca://player/status/gas.png"
                }
                PlayerState.Solid -> {
                    resource = "euca://player/status/solid.png"
                }
                PlayerState.Liquid -> {
                    resource = "euca://player/status/liquid.png"
                }
            }

            align = Align.default()
        }
    }

    bar("health_bar", Align.Center + 20.0)

    bar("energy_bar", Align.Center - 20.0)
}

bar(val objectName: String, align: Align = Align.default()): DynamicBoundingBox {
    Rectangle {
        name = objectName + " backing"

        size = Vector2d(30.0, 10.0)

        align = align

        style {
            fill = null
            stroke = Stroke(
                colour = Colour.BLACK,
                width = 1.0
            )
        }
    }

    Rectangle {
        name = objectName + " fill"

        size = Vector2d(Player.health.percentage() * 30.0, 10.0)

        align = align

        style {
            fill = Fill(
                colour = when (Player.currentZone) {
                    CurrentZone.Freezing -> Colour.DARK_BLUE
                    CurrentZone.Cold -> Colour.LIGHT_BLUE
                    CurrentZone.Normal -> Colour.LIGHT_GREY
                    CurrentZone.Hot -> Colour.ORANGE
                    CurrentZone.Boiling -> Colour.SCARLET_RED
                }
            )
            stroke = null
        }
    }
}