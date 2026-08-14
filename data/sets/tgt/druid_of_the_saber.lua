local card = {
    api_version = 1, id = "AT_042", name = "Druid of the Saber",
    text = "[x]<b>Choose One -</b> Transform\ninto a 2/1 with <b>Charge</b>;\nor a 3/2 with <b>Stealth</b>.",
    set = "TGT", type = "minion", class = "druid", rarity = "common",
    cost = 2, attack = 2, health = 1, keywords = { "choose_one" },
}

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { label = "Transform into a 2/1 with Charge", value = 1 },
        { label = "Transform into a 3/2 with Stealth", value = 2 },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    ctx:transform(self, choice == 1 and "AT_042t" or "AT_042t2")
end

function card.on_choose_multiple(ctx, self) ctx:transform(self, "OG_044c") end

card.tokens = {
    { id = "AT_042a", name = "Lion Form", text = "<b>Charge</b>", set = "TGT",
      type = "minion", class = "druid", rarity = "common", cost = 2, attack = 2,
      health = 1, tags = { "beast" }, keywords = { "charge" } },
    { id = "AT_042b", name = "Panther Form", text = "<b>Stealth</b>", set = "TGT",
      type = "minion", class = "druid", rarity = "common", cost = 2, attack = 3,
      health = 2, tags = { "beast" }, keywords = { "stealth" } },
    { id = "AT_042t", name = "Druid of the Saber", text = "<b>Charge</b>", set = "TGT",
      type = "minion", class = "druid", rarity = "common", cost = 2, attack = 2,
      health = 1, tags = { "beast" }, keywords = { "charge" } },
    { id = "AT_042t2", name = "Druid of the Saber", text = "<b>Stealth</b>", set = "TGT",
      type = "minion", class = "druid", rarity = "common", cost = 2, attack = 3,
      health = 2, tags = { "beast" }, keywords = { "stealth" } },
}

return card
