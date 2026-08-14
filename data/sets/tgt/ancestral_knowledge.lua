return {
    api_version = 1, id = "AT_053", name = "Ancestral Knowledge",
    text = "Draw 2 cards. <b>Overload:</b> (1)", set = "TGT", type = "spell",
    class = "shaman", rarity = "common", cost = 2,
    keywords = { "overload" }, keyword_params = { overload = 1 },
    on_play = function(ctx, self) ctx:draw(ctx:controller(self), 2) end,
}
