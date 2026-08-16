local card = {
    api_version = 1, id = "ICC_836", name = "Breath of Sindragosa",
    text = "Deal $2 damage to a random enemy minion and <b>Freeze</b> it.",
    set = "ICECROWN", type = "spell", class = "mage", rarity = "common",
    spell_school = "frost", cost = 1,
}

function card.on_play(ctx, self)
    local enemies, opponent = {}, ctx:opponent(ctx:controller(self))
    for _, minion in ipairs(ctx:minions()) do
        if ctx:controller(minion) == opponent then enemies[#enemies + 1] = minion end
    end
    if #enemies > 0 then ctx:random_entity(enemies, "breath_target_chosen") end
end

function card.breath_target_chosen(ctx, self, target)
    cardlib.effects.damage(ctx, target, 2)
    ctx:continue_with_entity("breath_freeze_survivor", target)
end

function card.breath_freeze_survivor(ctx, self, target)
    if ctx:entity(target).zone == "board" then ctx:freeze(target) end
end

return card
