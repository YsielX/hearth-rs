local card = {
    api_version = 1, id = "AT_004", name = "Arcane Blast",
    text = "Deal $2 damage to a minion. This spell gets double bonus from <b>Spell Damage</b>.",
    set = "TGT", type = "spell", class = "mage", rarity = "epic",
    spell_school = "arcane", cost = 1, target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    triggers = {
        {
            event = "damaged", timing = "before", active_zones = { "graveyard" },
            condition = function(ctx, self, event) return event.source == self end,
            effect = function(ctx, self, event)
                local bonus = 0
                for _, minion in ipairs(ctx:friendly_minions(self)) do
                    bonus = bonus + ctx:entity(minion).spell_damage
                end
                cardlib.effects.set_event_amount(ctx, event, event.amount + bonus)
            end,
        },
    },
}

function card.on_play(ctx, self, target) cardlib.effects.damage(ctx, target, 2) end
return card
