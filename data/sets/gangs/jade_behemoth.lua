local card = {
    api_version = 1,
    id = "CFM_343",
    name = "Jade Behemoth",
    text = "[x]<b>Taunt</b>\n<b>Battlecry:</b> Summon a{1}\n{0} <b>Jade Golem</b>.",
    set = "GANGS",
    type = "minion",
    class = "druid",
    rarity = "common",
    cost = 5,
    attack = 3,
    health = 6,
    keywords = { "taunt", "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    ctx:increment_player_data(player, "jade_golem_count", 1)
    ctx:continue_with("summon_jade_golem")
end

function card.summon_jade_golem(ctx, self)
    local player = ctx:controller(self)
    local size = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    cardlib.effects.summon_with_base_stats(ctx, player, "CFM_712_t01", size, size)
end

return card
