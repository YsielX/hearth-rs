local card = {
    api_version = 1,
    id = "TTN_457",
    name = "Eulogizer",
    text = "[x]<b>Battlecry:</b> Spend 3 <b>Corpses</b>\nto deal 3 damage.\n<b>Forge:</b> Gain them instead.",
    set = "TITANS",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "undead" },
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
    keywords = { "battlecry", "forge" },
}

card.action_effects = {
    forge = function(ctx, self)
        cardlib.effects.transform(ctx, self, "TTN_457t")
    end,
}

function card.on_battlecry(ctx, self, target)
    ctx:set_data(self, "eulogizer_target", target)
    ctx:spend_resource_and_continue(ctx:controller(self), "corpses", 3, 3, "deal_eulogy_damage")
end

function card.deal_eulogy_damage(ctx, self, spent)
    if spent > 0 then
        cardlib.effects.damage_ignoring_spell_damage(ctx, ctx:get_data(self, "eulogizer_target"), 3)
    end
end

card.tokens = {{
    id = "TTN_457t",
    name = "Eulogizer",
    text = "<b>Forged</b>\n<b>Battlecry:</b> Gain 3 <b>Corpses</b>. Deal 3 damage.",
    set = "TITANS",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    collectible = false,
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "undead" },
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self, target)
        ctx:gain_resource(ctx:controller(self), "corpses", 3)
        cardlib.effects.damage_ignoring_spell_damage(ctx, target, 3)
    end,
}}

return card
