return {
    api_version = 1,
    id = "KAR_029",
    name = "Runic Egg",
    text = "<b>Deathrattle:</b> Draw a card.",
    set = "KARA",
    type = "minion",
    rarity = "common",
    cost = 1,
    attack = 0,
    health = 2,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
}
