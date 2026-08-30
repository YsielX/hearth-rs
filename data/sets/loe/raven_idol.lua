local card = {
    api_version = 1, id = "LOE_115", name = "Raven Idol",
    text = "<b>Choose One -</b>\n<b>Discover</b> a minion; or <b>Discover</b> a spell.",
    set = "LOE", type = "spell", class = "druid", rarity = "common", cost = 1,
    keywords = { "choose_one", "discover" },
}

local function pool(ctx, self, wanted)
    local player = ctx:controller(self)
    local class = ctx:player(player).class
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == wanted
            and card_id ~= "LOE_115"
            and (definition.class == "neutral" or definition.class == class) then
            result[#result + 1] = card_id
        end
    end
    return result
end

local function discover_minion(ctx, self, hook)
    ctx:discover_cards(ctx:controller(self), "Discover a minion", pool(ctx, self, "minion"), 3, hook)
end

local function discover_spell(ctx, self, hook)
    ctx:discover_cards(ctx:controller(self), "Discover a spell", pool(ctx, self, "spell"), 3, hook)
end

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { label = "Discover a minion", value = 1 },
        { label = "Discover a spell", value = 2 },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    if choice == 1 then discover_minion(ctx, self, "receive_card")
    else discover_spell(ctx, self, "receive_card") end
end

function card.on_choose_multiple(ctx, self)
    discover_minion(ctx, self, "receive_minion_then_discover_spell")
end

function card.receive_card(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

function card.receive_minion_then_discover_spell(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
    discover_spell(ctx, self, "receive_card")
end

return card
