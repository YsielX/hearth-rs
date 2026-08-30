local card = {
    api_version = 1, id = "CFM_940", name = "I Know a Guy",
    text = "<b>Discover</b> a <b>Taunt</b> minion. Give it +1/+2.", set = "GANGS",
    type = "spell", class = "warrior", rarity = "common", cost = 1,
    keywords = { "discover" },
}
local function contains(values, wanted)
    for _, value in ipairs(values or {}) do if value == wanted then return true end end
    return false
end
function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local player_class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        local eligible_class = definition.class == player_class
            or (definition.class == "neutral" and (#(definition.classes or {}) == 0
                or contains(definition.classes, player_class)))
        if eligible_class and definition.type == "minion" and contains(definition.keywords, "taunt") then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then ctx:discover_cards(player, "Choose a Taunt minion", pool, 3, "receive_taunt") end
end
function card.receive_taunt(ctx, self, card_id) cardlib.effects.give_card(ctx, ctx:controller(self), card_id) end
card.triggers = {{
    event = "card_created", timing = "after", active_zones = { "graveyard" },
    condition = function(ctx, self, event) return event.source == self end,
    effect = function(ctx, self, event) cardlib.effects.buff(ctx, event.entity, 1, 2) end,
}}
return card
