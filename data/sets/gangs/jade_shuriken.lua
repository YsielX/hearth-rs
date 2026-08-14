local card = {
    api_version = 1, id = "CFM_690", name = "Jade Shuriken",
    text = "Deal $3 damage.\n<b>Combo:</b> Summon a{1} {0} <b>Jade Golem</b>.",
    set = "GANGS", type = "spell", class = "rogue", rarity = "common",
    cost = 2, target_mode = "required", keywords = { "combo" },
    targets = function(ctx, self) return ctx:all_characters() end,
}
function card.on_play(ctx, self, target)
    if not ctx:combo_active(self) then ctx:damage(target, 3) end
end
function card.on_combo(ctx, self, target)
    ctx:damage(target, 3)
    local player = ctx:controller(self)
    ctx:increment_player_data(player, "jade_golem_count", 1)
    ctx:continue_with("summon_jade_shuriken_golem")
end
function card.summon_jade_shuriken_golem(ctx, self)
    local player = ctx:controller(self)
    local n = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    ctx:summon_with_base_stats(player, "CFM_712_t01", n, n)
end
return card
