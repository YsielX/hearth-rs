return {
    api_version = 1,
    id = "ULD_208",
    name = "Khartut Defender",
    text = "[x]<b>Taunt</b>, <b>Reborn</b>\n<b>Deathrattle:</b> Restore #3\nHealth to your hero.",
    set = "ULDUM",
    type = "minion",
    cost = 6,
    attack = 3,
    health = 4,
    tags = { "undead" },
    keywords = { "taunt", "deathrattle", "reborn" },
    on_deathrattle = function(ctx, self)
        local player = ctx:controller(self)
        ctx:heal(ctx:player(player).hero, 3)
    end,
}
