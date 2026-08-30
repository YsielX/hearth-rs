local card = {
    api_version = 1,
    id = "TIME_000",
    name = "Semi-Stable Portal",
    text = "<b>Rewind</b>\nAdd a random minion\nto your hand. It costs\n(3) less.",
    set = "TIME_TRAVEL",
    type = "spell",
    class = "mage",
    spell_school = "arcane",
    rarity = "rare",
    cost = 2,
    keywords = { "rewind" },
    triggers = {
        {
            event = "card_created",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return event.source == self and event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                cardlib.effects.modify(ctx, event.entity, {
                    stat = "cost",
                    operation = "add",
                    value = -3,
                })
            end,
        },
    },
}

local function minion_pool(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(card_id).type == "minion" then
            result[#result + 1] = card_id
        end
    end
    return result
end

local function keep(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

function card.on_rewind(ctx, self)
    ctx:random_value(minion_pool(ctx), "portal_roll")
end

function card.portal_roll(ctx, self, card_id)
    local name = ctx:card_definition(card_id).name
    ctx:choose_options(ctx:controller(self), "Keep this timeline or Rewind?", {
        { label = "Keep: " .. name, value = { action = "keep", card = card_id } },
        { label = "Rewind", value = { action = "rewind" } },
    }, "portal_choice")
end

function card.portal_choice(ctx, self, choice)
    if choice.action == "keep" then
        keep(ctx, self, choice.card)
    else
        ctx:random_value(minion_pool(ctx), "portal_rewound")
    end
end

function card.portal_rewound(ctx, self, card_id)
    keep(ctx, self, card_id)
end

return card
