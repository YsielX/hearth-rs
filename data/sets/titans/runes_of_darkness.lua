local card = {
    api_version = 1,
    id = "YOG_511",
    name = "Runes of Darkness",
    text = "<b>Discover</b> a weapon. Spend 3 <b>Corpses</b> to give it +1/+1.",
    set = "TITANS",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    spell_school = "shadow",
    cost = 1,
    rune_cost = { blood = 1 },
    keywords = { "discover" },
}

local function supports_class(definition, player_class)
    if definition.class == "neutral" or definition.class == player_class then return true end
    for _, class in ipairs(definition.classes or {}) do
        if class == player_class then return true end
    end
    return false
end

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local player_class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "weapon" and supports_class(definition, player_class) then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then
        ctx:discover_cards(
            player,
            ctx:localize("Discover a weapon", "发现一张武器牌", "發現一張武器牌"),
            pool,
            3,
            "receive_weapon"
        )
    end
end

function card.receive_weapon(ctx, self, card_id)
    local player = ctx:controller(self)
    cardlib.effects.give_card(ctx, player, card_id)
    ctx:spend_resource_and_continue(player, "corpses", 3, 3, "buff_created_weapon")
end

function card.buff_created_weapon(ctx, self, spent)
    if spent == 0 then return end
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:get_data(entity, "runes_of_darkness_created") == 1 then
            ctx:set_data(entity, "runes_of_darkness_created", 0)
            cardlib.effects.buff(ctx, entity, 1, 1)
            return
        end
    end
end

card.triggers = {{
    event = "card_created",
    timing = "after",
    active_zones = { "graveyard" },
    condition = function(ctx, self, event)
        return event.source == self
    end,
    effect = function(ctx, self, event)
        ctx:set_data(event.entity, "runes_of_darkness_created", 1)
    end,
}}

return card
