return {
    api_version = 1,
    id = "BT_491", rarity = "common",
    name = "Spectral Sight",
    text = "[x]Draw a card.\n<b>Outcast:</b> Draw another.",
    set = "BLACK_TEMPLE",
    type = "spell",
    class = "demon_hunter",
    cost = 2,
    keywords = { "outcast" },
    on_play = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
    on_outcast = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
}
