local card = {
    api_version = 1,
    id = "DRG_050", rarity = "common",
    name = "Devoted Maniac",
    text = "<b>Rush</b>\n<b>Battlecry:</b> <b>Invoke</b> Galakrond.",
    set = "DRAGONS",
    type = "minion",
    cost = 4,
    attack = 2,
    health = 2,
    keywords = { "rush", "battlecry", "invoke" },
}

function card.on_battlecry(ctx, self) end

local function class_minions(ctx, class)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.class == class then
            result[#result + 1] = card_id
        end
    end
    return result
end

function card.on_invoke(ctx, self)
    local player = ctx:controller(self)
    local class = ctx:player(player).class
    if class == "warlock" then
        ctx:summon(player, "DRG_238t12t2")
        ctx:summon(player, "DRG_238t12t2")
    elseif class == "shaman" then
        ctx:summon(player, "DRG_238t14t3")
    elseif class == "warrior" then
        ctx:buff_until_end_of_turn(ctx:player(player).hero, 3, 0)
    elseif class == "priest" then
        local candidates = class_minions(ctx, "priest")
        if #candidates > 0 then ctx:random_value(candidates, "receive_invoked_card") end
    elseif class == "rogue" then
        ctx:random_value({ "DAL_613", "DAL_614", "DAL_615", "DAL_739", "DAL_741", "DRG_052", "ULD_616" }, "receive_invoked_card")
    end
end

function card.receive_invoked_card(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

card.tokens = {
    { id = "DRG_238t12t2", name = "Draconic Imp", text = "", set = "DRAGONS", type = "minion", class = "warlock", cost = 1, attack = 1, health = 1, tags = { "demon" } },
    { id = "DRG_238t14t3", name = "Windswept Elemental", text = "<b>Rush</b>", set = "DRAGONS", type = "minion", class = "shaman", cost = 2, attack = 2, health = 1, tags = { "elemental" }, keywords = { "rush" } },
    { id = "DAL_613", name = "Faceless Lackey", text = "<b>Battlecry:</b> Summon a random 2-Cost minion.", set = "DALARAN", type = "minion", cost = 1, attack = 1, health = 1 },
    { id = "DAL_614", name = "Kobold Lackey", text = "<b>Battlecry:</b> Deal 2 damage.", set = "DALARAN", type = "minion", cost = 1, attack = 1, health = 1 },
    { id = "DAL_615", name = "Witchy Lackey", text = "<b>Battlecry:</b> Transform a friendly minion into one that costs (1) more.", set = "DALARAN", type = "minion", cost = 1, attack = 1, health = 1 },
    { id = "DAL_739", name = "Goblin Lackey", text = "<b>Battlecry:</b> Give a friendly minion +1 Attack and <b>Rush</b>.", set = "DALARAN", type = "minion", cost = 1, attack = 1, health = 1 },
    { id = "DAL_741", name = "Ethereal Lackey", text = "<b>Battlecry:</b> <b>Discover</b> a spell.", set = "DALARAN", type = "minion", cost = 1, attack = 1, health = 1 },
    { id = "DRG_052", name = "Draconic Lackey", text = "<b>Battlecry:</b> <b>Discover</b> a Dragon.", set = "DRAGONS", type = "minion", cost = 1, attack = 1, health = 1 },
    { id = "ULD_616", name = "Titanic Lackey", text = "<b>Battlecry:</b> Give a friendly minion +2 Health and <b>Taunt</b>.", set = "ULDUM", type = "minion", cost = 1, attack = 1, health = 1 },
}

return card
