return {
    api_version = 1,
    id = "KAR_712",
    name = "Violet Illusionist",
    text = "During your turn, your hero is <b>Immune</b>.",
    set = "KARA",
    type = "minion",
    rarity = "common",
    cost = 3,
    attack = 4,
    health = 3,
    auras = {{
        active_zones = { "board" },
        keywords = { "immune" },
        targets = function(ctx, self)
            local player = ctx:controller(self)
            if ctx:active_player() == player then return { ctx:player(player).hero } end
            return {}
        end,
    }},
}
