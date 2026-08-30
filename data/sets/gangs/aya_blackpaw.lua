local card = {
    api_version = 1, id = "CFM_902", name = "Aya Blackpaw",
    text = "<b>Battlecry and Deathrattle:</b> Summon a{1} {0} <b>Jade Golem</b>.",
    set = "GANGS", type = "minion", classes = { "druid", "rogue", "shaman" },
    rarity = "legendary", cost = 6, attack = 6, health = 3,
    keywords = { "battlecry", "deathrattle" },
}
local function queue_jade(ctx, self)
    ctx:increment_player_data(ctx:controller(self), "jade_golem_count", 1)
    ctx:continue_with("summon_jade")
end
card.on_battlecry = queue_jade
card.on_deathrattle = queue_jade
function card.summon_jade(ctx, self)
    local player = ctx:controller(self)
    local size = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    cardlib.effects.summon_with_base_stats(ctx, player, "CFM_712_t01", size, size)
end
return card
