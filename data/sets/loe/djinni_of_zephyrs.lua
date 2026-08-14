local card = {
    api_version = 1,
    id = "LOE_053",
    name = "Djinni of Zephyrs",
    text = "After you cast a spell on another friendly minion, cast a copy of it on this one.",
    set = "LOE",
    type = "minion",
    rarity = "epic",
    cost = 5,
    attack = 4,
    health = 6,
    tags = { "elemental" },
}

card.triggers = {{
    event = "spell_cast",
    timing = "after",
    active_zones = { "board" },
    condition = function(ctx, self, event)
        if event.player ~= ctx:controller(self) or event.generated or event.target == nil
            or event.target == self then return false end
        local target = ctx:entity(event.target)
        return target.type == "minion" and target.controller == ctx:controller(self)
    end,
    effect = function(ctx, self, event)
        ctx:cast_spell_if_valid(
            ctx:controller(self), ctx:entity(event.entity).card_id, self
        )
    end,
}}

return card
