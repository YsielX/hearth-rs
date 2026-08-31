local card = {
    api_version = 1,
    id = "NEW1_007",
    name = "Starfall",
    text = "<b>Choose One -</b>\nDeal $5 damage to a minion; or $2 damage to all enemy minions.",
    set = "EXPERT1",
    type = "spell",
    class = "druid",
    rarity = "rare",
    spell_school = "arcane",
    cost = 5,
    keywords = { "choose_one" },
}

function card.on_choose_one(ctx, self)
    local options = {}
    if #ctx:minions() > 0 then
        options[#options + 1] = { card_id = "NEW1_007b", label = "Deal 5 damage to a minion" }
    end
    -- The area-damage mode remains a legal choice even on an empty board.
    options[#options + 1] = { card_id = "NEW1_007a", label = "Deal 2 damage to all enemy minions" }
    ctx:choose_options(ctx:controller(self), "Choose One", options, "chosen")
end

function card.chosen(ctx, self, choice)
    if choice == "NEW1_007b" then
        ctx:choose_entities(ctx:controller(self), "Choose a minion", ctx:minions(), "hit_selected")
    else
        cardlib.effects.damage_all(ctx, ctx:enemy_minions(self), 2)
    end
end

function card.hit_selected(ctx, self, target)
    cardlib.effects.damage(ctx, target, 5)
end

function card.on_choose_multiple(ctx, self)
    cardlib.effects.damage_all(ctx, ctx:enemy_minions(self), 2)
    local minions = ctx:minions()
    if #minions > 0 then
        ctx:choose_entities(ctx:controller(self), "Choose a minion", minions, "hit_selected")
    end
end

card.tokens = {
    { id = "NEW1_007a", spell_school = "arcane", name = "Stellar Drift", text = "Deal $2 damage to all enemy minions.", set = "EXPERT1", type = "spell", class = "druid", collectible = false, cost = 5 },
    { id = "NEW1_007b", spell_school = "arcane", name = "Starlord", text = "Deal $5 damage to a minion.", set = "EXPERT1", type = "spell", class = "druid", collectible = false, cost = 5 },
}

return card
