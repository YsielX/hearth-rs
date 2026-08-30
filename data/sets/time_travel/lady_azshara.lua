local card = {
    api_version = 1,
    id = "TIME_211", rarity = "legendary",
    name = "Lady Azshara",
    text = "[x]<b>Fabled</b>. <b>Choose One -</b>\nEmpower Zin-Azshari; or\nThe Well of Eternity. <i>(The\n  other gets destroyed!)</i>",
    set = "TIME_TRAVEL",
    type = "minion",
    class = "druid",
    cost = 5,
    attack = 5,
    health = 5,
    keywords = { "fabled", "choose_one" },
}

function card.on_fabled(ctx, self)
    local player = ctx:controller(self)
    cardlib.effects.shuffle_card_into_deck(ctx, player, "TIME_211t1")
    cardlib.effects.shuffle_card_into_deck(ctx, player, "TIME_211t2")
end

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { label = "Empower Zin-Azshari", value = 1 },
        { label = "Empower the Well of Eternity", value = 2 },
    }, "empower")
end

local function replace_in_zones(ctx, player, wanted, replacement)
    for _, zone in ipairs({ ctx:hand(player), ctx:deck(player), ctx:board(player) }) do
        for _, entity in ipairs(zone) do
            if ctx:entity(entity).card_id == wanted then cardlib.effects.transform(ctx, entity, replacement) end
        end
    end
end

local function destroy_in_zones(ctx, player, wanted)
    for _, zone in ipairs({ ctx:hand(player), ctx:deck(player), ctx:board(player) }) do
        for _, entity in ipairs(zone) do
            if ctx:entity(entity).card_id == wanted then ctx:move(entity, "graveyard") end
        end
    end
end

function card.empower(ctx, self, choice)
    local player = ctx:controller(self)
    if choice == 1 then
        replace_in_zones(ctx, player, "TIME_211t2", "TIME_211t2t")
        destroy_in_zones(ctx, player, "TIME_211t1")
    else
        replace_in_zones(ctx, player, "TIME_211t1", "TIME_211t1t")
        destroy_in_zones(ctx, player, "TIME_211t2")
    end
end

card.tokens = {
    { id = "TIME_211t1", name = "The Well of Eternity", text = "Fill your hand\nwith random\n<b>Temporary</b> spells.", set = "TIME_TRAVEL", type = "location", class = "druid", cost = 4, health = 3 },
    { id = "TIME_211t1t", name = "The Well of Eternity", text = "[x]Fill your hand\nwith random <b>Temporary</b>\nspells. They cast twice.", set = "TIME_TRAVEL", type = "location", class = "druid", cost = 4, health = 3 },
    { id = "TIME_211t2", name = "Zin-Azshari", text = "Summon a copy of a friendly minion.", set = "TIME_TRAVEL", type = "location", class = "druid", cost = 4, health = 3 },
    { id = "TIME_211t2t", name = "Zin-Azshari", text = "Summon a copy of a friendly minion with\nits stats doubled.", set = "TIME_TRAVEL", type = "location", class = "druid", cost = 4, health = 3 },
}

return card
