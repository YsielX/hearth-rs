local card = {
    api_version = 1,
    id = "CFM_713",
    name = "Jade Blossom",
    text = "Summon a{1} {0} <b>Jade Golem</b>. Gain an empty Mana Crystal.",
    set = "GANGS",
    type = "spell",
    class = "druid",
    rarity = "common",
    spell_school = "nature",
    cost = 3,
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    ctx:increment_player_data(player, "jade_golem_count", 1)
    ctx:continue_with("summon_jade_golem")
end

function card.summon_jade_golem(ctx, self)
    local player = ctx:controller(self)
    local size = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    cardlib.effects.summon_with_base_stats(ctx, player, "CFM_712_t01", size, size)
    ctx:gain_mana_crystals(player, 1, false)
end

return card
