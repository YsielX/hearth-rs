local card = {
    api_version = 1,
    id = "GVG_045",
    name = "Imp-losion",
    text = "Deal $2-$4 damage to a minion. Summon a 1/1 Imp for each damage dealt.",
    set = "GVG",
    type = "spell",
    class = "warlock",
    rarity = "rare",
    spell_school = "fel",
    cost = 4,
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return event.source == self and event.amount > 0
            end,
            effect = function(ctx, self, event)
                local player = ctx:controller(self)
                for _ = 1, event.amount do ctx:summon(player, "GVG_045t") end
            end,
        },
    },
}

function card.on_play(ctx, self, target)
    ctx:set_data(self, "target", target)
    ctx:random_value({ 2, 3, 4 }, "deal_implosion_damage")
end

function card.deal_implosion_damage(ctx, self, amount)
    ctx:damage(ctx:get_data(self, "target"), amount)
end

card.tokens = {
    {
        id = "GVG_045t",
        name = "Imp",
        text = "",
        set = "GVG",
        type = "minion",
        class = "warlock",
        cost = 1,
        attack = 1,
        health = 1,
        tags = { "demon" },
    },
}

return card
