local card = {
    api_version = 1, id = "CFM_691", name = "Jade Swarmer",
    text = "<b>Stealth</b>\n<b>Deathrattle:</b> Summon a{1} {0} <b>Jade Golem</b>.",
    set = "GANGS", type = "minion", class = "rogue", rarity = "common",
    cost = 1, attack = 1, health = 1, keywords = { "stealth", "deathrattle" },
}
function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    ctx:increment_player_data(player, "jade_golem_count", 1)
    ctx:continue_with("summon_jade_swarmer_golem")
end
function card.summon_jade_swarmer_golem(ctx, self)
    local player = ctx:controller(self)
    local n = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    ctx:summon_with_base_stats(player, "CFM_712_t01", n, n)
end
return card
