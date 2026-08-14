local card = {
    api_version = 1,
    id = "LOOT_414",
    name = "Grand Archivist",
    text = "At the end of your turn, cast a spell from your deck <i>(targets chosen randomly)</i>.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "epic",
    cost = 8,
    attack = 4,
    health = 7,
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                local spells = {}
                for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
                    if ctx:entity(entity).type == "spell" then spells[#spells + 1] = entity end
                end
                if #spells > 0 then ctx:random_value(spells, "archivist_spell_chosen") end
            end,
        },
    },
}

function card.archivist_spell_chosen(ctx, self, spell)
    ctx:cast_existing_spell(spell, {
        skip_if_invalid = true,
        random_target = true,
        choice_policy = "random",
    })
end

return card
