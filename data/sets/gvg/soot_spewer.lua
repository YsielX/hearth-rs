local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags) do
        if tag == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_123",
    name = "Soot Spewer",
    text = "<b>Spell Damage +1</b>\n<b>Battlecry:</b> If you control\nanother Mech, get a random Fire spell.",
    set = "GVG",
    type = "minion",
    class = "mage",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 4,
    tags = { "mech" },
    keywords = { "spell_damage", "battlecry" },
    keyword_params = { spell_damage = 1 },
}

function card.on_battlecry(ctx, self)
    local controls_other_mech = false
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self and has_tag(ctx:card_definition(ctx:entity(minion).card_id), "mech") then
            controls_other_mech = true
            break
        end
    end
    if not controls_other_mech then return end

    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "spell" and definition.spell_school == "fire" then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "receive_fire_spell") end
end

function card.receive_fire_spell(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
