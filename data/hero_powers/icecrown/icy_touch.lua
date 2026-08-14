local power = {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_833h",
    name = "Icy Touch",
    text = "Deal $1 damage. If this kills a minion, summon a Water Elemental.",
    set = "ICECROWN",
    class = "neutral",
    cost = 2,
    target_mode = "required",
    targets = function(ctx, self) return ctx:characters() end,
}

function power.on_play(ctx, self, target)
    local was_minion = ctx:entity(target).type == "minion"
    ctx:damage(target, 1)
    if was_minion then ctx:continue_with_entity("after_damage", target) end
end

function power.after_damage(ctx, self, target)
    if ctx:entity(target).zone == "graveyard" then
        ctx:summon(ctx:controller(self), "ICC_833t")
    end
end

return power
