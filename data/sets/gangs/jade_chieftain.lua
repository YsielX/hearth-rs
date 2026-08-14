local card = {
    api_version = 1, id = "CFM_312", name = "Jade Chieftain",
    text = "<b>Battlecry:</b> Summon a{1} {0} <b>Jade Golem</b>. Give it <b>Taunt</b>.",
    set = "GANGS", type = "minion", class = "shaman", rarity = "common",
    cost = 6, attack = 5, health = 5, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    ctx:increment_player_data(ctx:controller(self), "jade_golem_count", 1)
    ctx:continue_with("summon_jade")
end
function card.summon_jade(ctx, self)
    local player = ctx:controller(self)
    local size = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    ctx:summon_with_base_stats(player, "CFM_712_t01", size, size, { "taunt" })
end
return card
