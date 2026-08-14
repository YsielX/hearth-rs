local card = {
    api_version = 1, id = "CFM_715", name = "Jade Spirit",
    text = "<b>Battlecry:</b> Summon a{1} {0} <b>Jade Golem</b>.", set = "GANGS",
    type = "minion", classes = { "druid", "rogue", "shaman" }, rarity = "common",
    cost = 4, attack = 3, health = 3, tags = { "elemental" }, keywords = { "battlecry" },
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
card.tokens = {
    { id = "CFM_712_t01", name = "Jade Golem", text = "", set = "GANGS", type = "minion", cost = 1, attack = 1, health = 1 },
    { id = "CFM_712_t04", name = "Jade Golem", text = "", set = "GANGS", type = "minion", cost = 4, attack = 4, health = 4 },
    { id = "CFM_712_t07", name = "Jade Golem", text = "", set = "GANGS", type = "minion", cost = 7, attack = 7, health = 7 },
    { id = "CFM_712_t20", name = "Jade Golem", text = "", set = "GANGS", type = "minion", cost = 10, attack = 20, health = 20 },
}
return card
