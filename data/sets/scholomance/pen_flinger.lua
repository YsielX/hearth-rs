return {
    api_version = 1,
    id = "SCH_248",
    name = "Pen Flinger",
    text = "[x]<b>Battlecry:</b> Deal 1 damage\nto a minion.\n <b><b>Spellburst</b>:</b> Return this\nto your hand.",
    set = "SCHOLOMANCE",
    type = "minion",
    cost = 1,
    attack = 1,
    health = 1,
    keywords = { "battlecry", "spellburst" },
    target_mode = "required_if_available",

    targets = function(ctx, self)
        return ctx:minions()
    end,

    on_battlecry = function(ctx, self, target)
        if target ~= nil then
            cardlib.effects.damage(ctx, target, 1)
        end
    end,

    on_spellburst = function(ctx, self)
        ctx:move(self, "hand")
    end,
}
