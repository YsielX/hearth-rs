local card = {
    api_version = 1, id = "CFM_717", name = "Jade Claws",
    text = "<b>Battlecry:</b> Summon a{1} {0} <b>Jade Golem</b>.\n<b><b>Overload</b>:</b> (1)0<b>Battlecry:</b> Summon a <b>Jade Golem</b>.\n<b><b>Overload</b>:</b> (1)",
    set = "GANGS", type = "weapon", class = "shaman", rarity = "rare",
    cost = 2, attack = 2, health = 2, keywords = { "battlecry", "overload" },
    keyword_params = { overload = 1 },
}
function card.on_battlecry(ctx, self)
    ctx:increment_player_data(ctx:controller(self), "jade_golem_count", 1)
    ctx:continue_with("summon_jade")
end
function card.summon_jade(ctx, self)
    local player = ctx:controller(self)
    local size = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    ctx:summon_with_base_stats(player, "CFM_712_t01", size, size)
end
return card
