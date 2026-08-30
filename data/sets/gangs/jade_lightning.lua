local card = {
    api_version = 1, id = "CFM_707", spell_school = "nature", name = "Jade Lightning",
    text = "Deal $3 damage. Summon a{1} {0} <b>Jade Golem</b>.", set = "GANGS",
    type = "spell", class = "shaman", rarity = "common", cost = 3,
    target_mode = "required", targets = function(ctx) return ctx:characters() end,
}
function card.on_play(ctx, self, target)
    cardlib.effects.damage(ctx, target, 3)
    ctx:continue_with("queue_jade")
end
function card.queue_jade(ctx, self)
    ctx:increment_player_data(ctx:controller(self), "jade_golem_count", 1)
    ctx:continue_with("summon_jade")
end
function card.summon_jade(ctx, self)
    local player = ctx:controller(self)
    local size = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    cardlib.effects.summon_with_base_stats(ctx, player, "CFM_712_t01", size, size)
end
return card
